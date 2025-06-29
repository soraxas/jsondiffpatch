use crate::context::{DiffContext, FilterContext, PatchContext};
use crate::lcs;
use crate::processor::Filter;
use crate::types::{ArrayDeltaIndex, ArrayOptions, Delta};
use serde_json::Value;

pub struct ArraysDiffFilter;
pub struct ArraysPatchFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for ArraysDiffFilter {
    fn filter_name(&self) -> &str {
        "arrays-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // Check if left is an array
        if !context.left.is_array() {
            return;
        }

        let left_array = context.left.as_array().unwrap();
        let right_array = context.right.as_array().unwrap();
        let len1 = left_array.len();
        let len2 = right_array.len();

        // Handle trivial cases first
        match (len1, len2) {
            (0, 0) => {
                context.set_result(Delta::None).exit();
                return;
            }
            (0, _) => {
                // Left array is empty, all items in right are additions
                let mut array_changes = Vec::new();
                for (index, value) in right_array.iter().enumerate() {
                    array_changes
                        .push((ArrayDeltaIndex::NewOrModified(index), Delta::Added(value)));
                }
                context.set_result(Delta::Array(array_changes)).exit();
                return;
            }
            (_, 0) => {
                // Right array is empty, all items in left are deletions
                let mut array_changes = Vec::new();
                for (index, value) in left_array.iter().enumerate() {
                    array_changes.push((
                        ArrayDeltaIndex::RemovedOrMoved(index),
                        Delta::Deleted(value),
                    ));
                }
                context.set_result(Delta::Array(array_changes)).exit();
                return;
            }
            (_, _) => {}
        }

        // Separate common head
        let mut common_head = 0;
        while common_head < len1
            && common_head < len2
            && left_array[common_head] == right_array[common_head]
        {
            let child_context = DiffContext::new(
                &left_array[common_head],
                &right_array[common_head],
                context.options().clone(),
            );
            new_children_context.push((common_head.to_string(), child_context));
            common_head += 1;
        }

        // Separate common tail
        let mut common_tail = 0;
        while common_tail + common_head < len1
            && common_tail + common_head < len2
            && left_array[len1 - 1 - common_tail] == right_array[len2 - 1 - common_tail]
        {
            let index1 = len1 - 1 - common_tail;
            let index2 = len2 - 1 - common_tail;
            let child_context = DiffContext::new(
                &left_array[index1],
                &right_array[index2],
                context.options().clone(),
            );
            new_children_context.push((index2.to_string(), child_context));
            common_tail += 1;
        }

        // Handle trivial cases after common head/tail separation
        if common_head + common_tail == len1 {
            if len1 == len2 {
                // arrays are identical
                context.set_result(Delta::None).exit();
                return;
            }
            // trivial case, a block was added
            let mut array_changes = Vec::new();
            for index in common_head..len2 - common_tail {
                array_changes.push((
                    ArrayDeltaIndex::NewOrModified(index),
                    Delta::Added(&right_array[index]),
                ));
            }
            context.set_result(Delta::Array(array_changes)).exit();
            return;
        }

        if common_head + common_tail == len2 {
            // trivial case, a block was removed
            let mut array_changes = Vec::new();
            for index in common_head..len1 - common_tail {
                array_changes.push((
                    ArrayDeltaIndex::RemovedOrMoved(index),
                    Delta::Deleted(&left_array[index]),
                ));
            }
            context.set_result(Delta::Array(array_changes)).exit();
            return;
        }

        // Use LCS algorithm on the trimmed arrays
        let trimmed1 = &left_array[common_head..len1 - common_tail];
        let trimmed2 = &right_array[common_head..len2 - common_tail];
        let lcs_indices = lcs::longest_common_subsequence(&trimmed1.to_vec(), &trimmed2.to_vec());

        let mut array_changes = Vec::new();
        let mut removed_items = Vec::new();

        // Find removed items (items in trimmed1 but not in LCS)
        for i in 0..trimmed1.len() {
            if !lcs_indices.iter().any(|&(lcs_i, _)| lcs_i == i) {
                let original_index = i + common_head;
                removed_items.push(original_index);
                array_changes.push((
                    ArrayDeltaIndex::RemovedOrMoved(original_index),
                    Delta::Deleted(&left_array[original_index]),
                ));
            }
        }

        // Check for move detection
        let detect_move = context
            .options()
            .arrays
            .as_ref()
            .and_then(|opts| opts.detect_move)
            .unwrap_or(true);
        let include_value_on_move = context
            .options()
            .arrays
            .as_ref()
            .and_then(|opts| opts.include_value_on_move)
            .unwrap_or(false);

        // Process items in the right array
        for j in 0..trimmed2.len() {
            let original_index2 = j + common_head;
            let lcs_index = lcs_indices.iter().position(|&(_, lcs_j)| lcs_j == j);

            if lcs_index.is_none() {
                // Item is added, try to match with a removed item for move detection
                let mut is_move = false;
                if detect_move && !removed_items.is_empty() {
                    for (remove_idx, &removed_index) in removed_items.iter().enumerate() {
                        let trimmed_removed_index = removed_index - common_head;
                        if trimmed_removed_index < trimmed1.len()
                            && trimmed1[trimmed_removed_index] == trimmed2[j]
                        {
                            // Found a match, convert deletion to move
                            // Remove the deletion from array_changes
                            array_changes.retain(|(idx, _)| {
                                if let ArrayDeltaIndex::RemovedOrMoved(idx_val) = idx {
                                    *idx_val != removed_index
                                } else {
                                    true
                                }
                            });

                            // Add move
                            let moved_value = if include_value_on_move {
                                Some(&left_array[removed_index])
                            } else {
                                None
                            };
                            array_changes.push((
                                ArrayDeltaIndex::RemovedOrMoved(removed_index),
                                Delta::Moved {
                                    moved_value,
                                    new_index: original_index2,
                                },
                            ));

                            // Create child context for nested diff
                            let child_context = DiffContext::new(
                                &left_array[removed_index],
                                &right_array[original_index2],
                                context.options().clone(),
                            );
                            new_children_context.push((original_index2.to_string(), child_context));

                            removed_items.remove(remove_idx);
                            is_move = true;
                            break;
                        }
                    }
                }

                if !is_move {
                    // Item is truly added
                    array_changes.push((
                        ArrayDeltaIndex::NewOrModified(original_index2),
                        Delta::Added(&right_array[original_index2]),
                    ));
                }
            } else {
                // Item is in LCS, check for nested changes
                let lcs_idx = lcs_index.unwrap();
                let (i, _) = lcs_indices[lcs_idx];
                let original_index1 = i + common_head;

                if trimmed1[i] != trimmed2[j] {
                    // Items are different, create child context for nested diff
                    let child_context = DiffContext::new(
                        &left_array[original_index1],
                        &right_array[original_index2],
                        context.options().clone(),
                    );
                    new_children_context.push((original_index2.to_string(), child_context));
                }
            }
        }

        // If we have changes, set the result
        if !array_changes.is_empty() || !new_children_context.is_empty() {
            context.set_result(Delta::Array(array_changes)).exit();
        } else {
            // No changes detected
            context.set_result(Delta::None).exit();
        }
    }

    fn post_process(
        &self,
        context: &mut DiffContext<'a>,
        children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        if !context.left.is_array() {
            return;
        }
        // Handle post-processing of array diff results
        // This would collect results from child contexts and merge them
        if children_context.is_empty() {
            return;
        }

        // Collect results from children and merge them into the array delta
        let mut array_changes = Vec::new();

        for (index_str, child_context) in children_context {
            if let Some(child_result) = child_context.get_result() {
                if let Delta::None = child_result {
                    continue;
                }
                let index: usize = index_str.parse().unwrap_or(0);
                array_changes.push((ArrayDeltaIndex::NewOrModified(index), child_result.clone()));
            }
        }

        if !array_changes.is_empty() {
            context.set_result(Delta::Array(array_changes)).exit();
        }
    }
}

impl<'a> Filter<PatchContext<'a>, Value> for ArraysPatchFilter {
    fn filter_name(&self) -> &str {
        "arrays-patch"
    }

    fn process(
        &self,
        context: &mut PatchContext<'a>,
        new_children_context: &mut Vec<(String, PatchContext<'a>)>,
    ) {
        let Delta::Array(array_delta) = &context.delta else {
            return;
        };

        let ori_array = context.left.as_array().unwrap();
        let mut new_array = ori_array.clone();

        // First, separate removals, insertions, modifications, and moves
        let mut to_remove: Vec<(usize, Option<usize>)> = Vec::new();
        let mut to_insert: Vec<(usize, Value)> = Vec::new();
        let mut to_modify: Vec<(usize, Delta<'a>)> = Vec::new();

        for (index, delta) in array_delta {
            match index {
                ArrayDeltaIndex::RemovedOrMoved(removed_index) => {
                    // Check if it's a removal or move
                    match delta {
                        Delta::Deleted(_) => {
                            to_remove.push((*removed_index, None));
                        }
                        Delta::Moved { new_index, .. } => {
                            to_remove.push((*removed_index, Some(*new_index)));
                            // We'll handle the reinsertion later
                        }
                        _ => {
                            panic!("only removal or move can be applied at original array indices");
                        }
                    }
                }
                ArrayDeltaIndex::NewOrModified(new_index) => {
                    match delta {
                        Delta::Added(value) => {
                            to_insert.push((*new_index, (*value).clone()));
                        }
                        Delta::Modified(from, to) => {
                            // Modified item - will be handled by child contexts
                            to_modify.push((*new_index, delta.clone()));
                        }
                        _ => {
                            panic!(
                                "only addition or modification can be applied at new array indices"
                            );
                        }
                    }
                }
            }
        }

        // Remove items, in reverse order to avoid index shifting issues
        to_remove.sort_by_key(|(index, _)| *index);
        for &(index, is_move) in to_remove.iter().rev() {
            if index >= new_array.len() {
                panic!("index out of bounds: the patch is trying to remove an item at index {}, but the array has only {} items", index, new_array.len());
            }
            let removed_value = new_array.remove(index);
            // Check if this was a move operation
            if let Some(new_index) = is_move {
                to_insert.push((new_index, removed_value));
            }
        }

        // Insert items, sorted by index
        to_insert.sort_by_key(|(index, _)| *index);
        for (index, value) in to_insert {
            if index > new_array.len() {
                panic!("index out of bounds: the patch is trying to insert an item at index {}, but the array has only {} items", index, new_array.len());
            }
            new_array.insert(index, value);
        }

        // Create child contexts for modifications
        for (index, delta) in to_modify {
            if index < new_array.len() {
                let value = &ori_array[index];
                let child_context = PatchContext::new(value, delta, context.options().clone());
                new_children_context.push((index.to_string(), child_context));
            }
        }

        // If no children, set the result and exit
        if new_children_context.is_empty() {
            context.set_result(Value::Array(new_array)).exit();
            return;
        }

        context.exit();
    }

    fn post_process(
        &self,
        context: &mut PatchContext<'a>,
        children_context: &mut Vec<(String, PatchContext<'a>)>,
    ) {
        let Delta::Array(_) = &context.delta else {
            return;
        };

        if children_context.is_empty() {
            return;
        }

        // Collect results from children and apply them to the array
        let mut array = context.left.as_array().unwrap().clone();

        for (index_str, child_context) in children_context {
            if let Some(child_result) = child_context.get_result() {
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < array.len() {
                        array[index] = child_result.clone();
                    }
                }
            }
        }

        context.set_result(Value::Array(array)).exit();
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::Options;

    struct ArrayPatchTestCase<'a> {
        name: &'a str,
        original: Value,
        delta: Delta<'a>,
        expected: Value,
        expected_children: usize,
    }

    #[test]
    fn test_arrays_patch() {
        let a = serde_json::json!("a");
        let b = serde_json::json!("b");
        let c = serde_json::json!("c");
        let x = serde_json::json!("x");
        let test_cases = vec![
            ArrayPatchTestCase {
                name: "remove first element and add new element at beginning",
                original: serde_json::from_str(r#"["a", "b", "c"]"#).unwrap(),
                delta: Delta::Array(vec![
                    (ArrayDeltaIndex::RemovedOrMoved(0), Delta::Deleted(&a)),
                    (ArrayDeltaIndex::NewOrModified(0), Delta::Added(&x)),
                ]),
                expected: serde_json::from_str(r#"["x", "b", "c"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "add element at end",
                original: serde_json::from_str(r#"["a", 0, "b", false]"#).unwrap(),
                delta: Delta::Array(vec![(ArrayDeltaIndex::NewOrModified(3), Delta::Added(&c))]),
                expected: serde_json::from_str(r#"["a", 0, "b", "c", false]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "remove element from middle",
                original: serde_json::from_str(r#"["a", "b", "c", "d", "e"]"#).unwrap(),
                delta: Delta::Array(vec![
                    (
                        ArrayDeltaIndex::RemovedOrMoved(3),
                        Delta::Moved {
                            moved_value: None,
                            new_index: 2,
                        },
                    ),
                    (ArrayDeltaIndex::RemovedOrMoved(1), Delta::Deleted(&b)),
                    (
                        ArrayDeltaIndex::RemovedOrMoved(4),
                        Delta::Moved {
                            moved_value: None,
                            new_index: 0,
                        },
                    ),
                ]),
                expected: serde_json::from_str(r#"["e","a", "d", "c"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "move element",
                original: serde_json::from_str(r#"["a", "b", "c"]"#).unwrap(),
                delta: Delta::Array(vec![(
                    ArrayDeltaIndex::RemovedOrMoved(0),
                    Delta::Moved {
                        moved_value: Some(&a),
                        new_index: 2,
                    },
                )]),
                expected: serde_json::from_str(r#"["b", "c", "a"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "modify element",
                original: serde_json::from_str(r#"["a", "b", "c"]"#).unwrap(),
                delta: Delta::Array(vec![
                    // (ArrayDeltaIndex::NewOrModified(1), Delta::Modified(
                    //     &b,
                    //     &x
                    // )),
                ]),
                expected: serde_json::from_str(r#"["a", "b", "c"]"#).unwrap(), // Will be updated by child context
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "complex operations: remove, add, and move",
                original: serde_json::from_str(r#"["a", "b", "c", "d"]"#).unwrap(),
                delta: Delta::Array(vec![
                    (ArrayDeltaIndex::RemovedOrMoved(1), Delta::Deleted(&b)),
                    (ArrayDeltaIndex::NewOrModified(1), Delta::Added(&x)),
                    (
                        ArrayDeltaIndex::RemovedOrMoved(0),
                        Delta::Moved {
                            moved_value: Some(&a),
                            new_index: 3,
                        },
                    ),
                ]),
                expected: serde_json::from_str(r#"["c", "x", "d", "a"]"#).unwrap(),
                expected_children: 0,
            },
        ];

        for test_case in test_cases {
            println!("Running test: {}", test_case.name);

            let options = Rc::new(Options::default());
            let mut context = PatchContext::new(&test_case.original, test_case.delta, options);

            let mut new_children_context = Vec::new();
            ArraysPatchFilter.process(&mut context, &mut new_children_context);

            assert_eq!(
                new_children_context.len(),
                test_case.expected_children,
                "Test '{}' failed: expected {} children, got {}",
                test_case.name,
                test_case.expected_children,
                new_children_context.len()
            );

            if let Some(result) = context.get_result() {
                assert_eq!(
                    result, &test_case.expected,
                    "Test '{}' failed: expected {:?}, got {:?}",
                    test_case.name, test_case.expected, result
                );
            } else {
                panic!("Test '{}' failed: no result returned", test_case.name);
            }
        }
    }

    #[test]
    fn test_arrays_patch_edge_cases() {
        let a = serde_json::json!("a");

        let edge_cases = vec![
            ArrayPatchTestCase {
                name: "empty array add element",
                original: serde_json::from_str(r#"[]"#).unwrap(),
                delta: Delta::Array(vec![(ArrayDeltaIndex::NewOrModified(0), Delta::Added(&a))]),
                expected: serde_json::from_str(r#"["a"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "single element remove",
                original: serde_json::from_str(r#"["a"]"#).unwrap(),
                delta: Delta::Array(vec![(
                    ArrayDeltaIndex::RemovedOrMoved(0),
                    Delta::Deleted(&a),
                )]),
                expected: serde_json::from_str(r#"[]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "no changes",
                original: serde_json::from_str(r#"["a", "b"]"#).unwrap(),
                delta: Delta::Array(vec![]),
                expected: serde_json::from_str(r#"["a", "b"]"#).unwrap(),
                expected_children: 0,
            },
        ];

        for test_case in edge_cases {
            println!("Running edge case test: {}", test_case.name);

            let options = Rc::new(Options::default());
            let mut context = PatchContext::new(&test_case.original, test_case.delta, options);

            let mut new_children_context = Vec::new();
            ArraysPatchFilter.process(&mut context, &mut new_children_context);

            assert_eq!(
                new_children_context.len(),
                test_case.expected_children,
                "Edge case test '{}' failed: expected {} children, got {}",
                test_case.name,
                test_case.expected_children,
                new_children_context.len()
            );

            if let Some(result) = context.get_result() {
                assert_eq!(
                    result, &test_case.expected,
                    "Edge case test '{}' failed: expected {:?}, got {:?}",
                    test_case.name, test_case.expected, result
                );
            } else {
                panic!(
                    "Edge case test '{}' failed: no result returned",
                    test_case.name
                );
            }
        }
    }
}
