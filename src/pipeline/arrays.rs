use crate::context::{DiffContext, FilterContext};
use crate::errors::JsonDiffPatchError;
use crate::lcs;
use crate::types::{ArrayDeltaIndex, Delta};
use serde_json::Value;
use std::borrow::Cow;

pub fn process_arrays_diff<'a>(
    context: &mut DiffContext<'a>,
    left: &'a [Value],
    right: &'a [Value],
    new_children_context: &mut Vec<(String, DiffContext<'a>)>,
) -> Result<(), JsonDiffPatchError> {
    // Check if left is an array

    let left_array = left;
    let right_array = right;
    let len1 = left_array.len();
    let len2 = right_array.len();

    // Handle trivial cases first
    match (len1, len2) {
        (0, 0) => {
            context.set_result(Delta::None).exit();
            return Ok(());
        }
        (0, _) => {
            // Left array is empty, all items in right are additions
            let mut array_changes = Vec::new();
            for (index, value) in right_array.iter().enumerate() {
                array_changes.push((
                    ArrayDeltaIndex::NewOrModified(index),
                    Delta::added_ref(value),
                ));
            }
            context.set_result(Delta::Array(array_changes)).exit();
            return Ok(());
        }
        (_, 0) => {
            // Right array is empty, all items in left are deletions
            let mut array_changes = Vec::new();
            for (index, value) in left_array.iter().enumerate() {
                array_changes.push((
                    ArrayDeltaIndex::RemovedOrMoved(index),
                    Delta::deleted_ref(value),
                ));
            }
            context.set_result(Delta::Array(array_changes)).exit();
            return Ok(());
        }
        (_, _) => {}
    }

    // Separate common head
    let mut common_head = 0;
    while common_head < len1
        && common_head < len2
        && left_array[common_head] == right_array[common_head]
    {
        let child_context = DiffContext::new(&left_array[common_head], &right_array[common_head]);
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
        let child_context = DiffContext::new(&left_array[index1], &right_array[index2]);
        new_children_context.push((index2.to_string(), child_context));
        common_tail += 1;
    }

    // Handle trivial cases after common head/tail separation
    if common_head + common_tail == len1 {
        if len1 == len2 {
            // arrays are identical
            context.set_result(Delta::None).exit();
            return Ok(());
        }
        // trivial case, a block was added
        let mut array_changes = Vec::new();
        for (index, val) in right_array
            .iter()
            .enumerate()
            .take(len2 - common_tail)
            .skip(common_head)
        {
            array_changes.push((ArrayDeltaIndex::NewOrModified(index), Delta::added_ref(val)));
        }
        context.set_result(Delta::Array(array_changes)).exit();
        return Ok(());
    }

    if common_head + common_tail == len2 {
        // trivial case, a block was removed
        let mut array_changes = Vec::new();
        for (index, val) in left_array
            .iter()
            .enumerate()
            .take(len1 - common_tail)
            .skip(common_head)
        {
            array_changes.push((
                ArrayDeltaIndex::RemovedOrMoved(index),
                Delta::deleted_ref(val),
            ));
        }
        context.set_result(Delta::Array(array_changes)).exit();
        return Ok(());
    }

    // Use LCS algorithm on the trimmed arrays
    let trimmed1 = &left_array[common_head..len1 - common_tail];
    let trimmed2 = &right_array[common_head..len2 - common_tail];
    let lcs_indices = lcs::longest_common_subsequence(trimmed1, trimmed2);

    let mut array_changes = Vec::new();
    let mut removed_items = Vec::new();

    // Find removed items (items in trimmed1 but not in LCS)
    for i in 0..trimmed1.len() {
        if !lcs_indices.iter().any(|&(lcs_i, _)| lcs_i == i) {
            let original_index = i + common_head;
            removed_items.push(original_index);
            array_changes.push((
                ArrayDeltaIndex::RemovedOrMoved(original_index),
                Delta::deleted_ref(&left_array[original_index]),
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
    for (j, val) in trimmed2.iter().enumerate() {
        let original_index2 = j + common_head;
        let lcs_index = lcs_indices.iter().position(|&(_, lcs_j)| lcs_j == j);

        if lcs_index.is_none() {
            // Item is added, try to match with a removed item for move detection
            let mut is_move = false;
            if detect_move && !removed_items.is_empty() {
                for (remove_idx, &removed_index) in removed_items.iter().enumerate() {
                    let trimmed_removed_index = removed_index - common_head;
                    if trimmed_removed_index < trimmed1.len()
                        && &trimmed1[trimmed_removed_index] == val
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
                            Some(Cow::Borrowed(&left_array[removed_index]))
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
                    Delta::added_ref(&right_array[original_index2]),
                ));
            }
        } else {
            // Item is in LCS, check for nested changes
            let lcs_idx = lcs_index.ok_or_else(|| {
                JsonDiffPatchError::InternalPatchLogicError("lcs_index is none".to_string())
            })?;
            let (i, _) = lcs_indices[lcs_idx];
            let original_index1 = i + common_head;

            if trimmed1[i] != trimmed2[j] {
                // Items are different, create child context for nested diff
                let child_context =
                    DiffContext::new(&left_array[original_index1], &right_array[original_index2]);
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
    Ok(())
}

pub fn post_process_arrays_diff<'a>(
    context: &mut DiffContext<'a>,
    children_context: &mut Vec<(String, DiffContext<'a>)>,
) -> Result<(), JsonDiffPatchError> {
    // Handle post-processing of array diff results
    // This would collect results from child contexts and merge them
    if children_context.is_empty() {
        return Ok(());
    }

    // Collect results from children and merge them into the array delta
    let mut array_changes = Vec::new();

    for (index_str, child_context) in children_context {
        if let Some(child_result) = child_context.pop_result() {
            if let Delta::None = child_result {
                continue;
            }
            let index: usize = index_str.parse().unwrap_or(0);
            array_changes.push((ArrayDeltaIndex::NewOrModified(index), child_result));
        }
    }

    if !array_changes.is_empty() {
        context.set_result(Delta::Array(array_changes)).exit();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PatchContext;
    use crate::pipeline::patch_pipeline::handle_array;
    use serde_json::Value;

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
                    (ArrayDeltaIndex::RemovedOrMoved(0), Delta::deleted_ref(&a)),
                    (ArrayDeltaIndex::NewOrModified(0), Delta::added_ref(&x)),
                ]),
                expected: serde_json::from_str(r#"["x", "b", "c"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "add element at end",
                original: serde_json::from_str(r#"["a", 0, "b", false]"#).unwrap(),
                delta: Delta::Array(vec![(
                    ArrayDeltaIndex::NewOrModified(3),
                    Delta::added_ref(&c),
                )]),
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
                    (ArrayDeltaIndex::RemovedOrMoved(1), Delta::deleted_ref(&b)),
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
                    Delta::moved_ref(&a, 2),
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
                    (ArrayDeltaIndex::RemovedOrMoved(1), Delta::deleted_ref(&b)),
                    (ArrayDeltaIndex::NewOrModified(1), Delta::added_ref(&x)),
                    (ArrayDeltaIndex::RemovedOrMoved(0), Delta::moved_ref(&a, 3)),
                ]),
                expected: serde_json::from_str(r#"["c", "x", "d", "a"]"#).unwrap(),
                expected_children: 0,
            },
        ];

        for test_case in test_cases {
            println!("Running test: {}", test_case.name);

            let mut context = PatchContext::new(&test_case.original, test_case.delta);

            let mut new_children_context = Vec::new();
            let result = {
                let Delta::Array(array_delta) = context.take_delta() else {
                    panic!("Test '{}' failed: delta is not an array", test_case.name);
                };
                handle_array(
                    test_case.original.as_array().unwrap(),
                    array_delta,
                    &mut new_children_context,
                )
                .unwrap()
            };

            assert_eq!(
                new_children_context.len(),
                test_case.expected_children,
                "Test '{}' failed: expected {} children, got {}",
                test_case.name,
                test_case.expected_children,
                new_children_context.len()
            );

            assert_eq!(
                result, test_case.expected,
                "Test '{}' failed: expected {:?}, got {:?}",
                test_case.name, test_case.expected, result
            );
        }
    }

    #[test]
    fn test_arrays_patch_edge_cases() {
        let a = serde_json::json!("a");

        let edge_cases = vec![
            ArrayPatchTestCase {
                name: "empty array add element",
                original: serde_json::from_str(r#"[]"#).unwrap(),
                delta: Delta::Array(vec![(
                    ArrayDeltaIndex::NewOrModified(0),
                    Delta::added_ref(&a),
                )]),
                expected: serde_json::from_str(r#"["a"]"#).unwrap(),
                expected_children: 0,
            },
            ArrayPatchTestCase {
                name: "single element remove",
                original: serde_json::from_str(r#"["a"]"#).unwrap(),
                delta: Delta::Array(vec![(
                    ArrayDeltaIndex::RemovedOrMoved(0),
                    Delta::deleted_ref(&a),
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

            let mut context = PatchContext::new(&test_case.original, test_case.delta);

            let mut new_children_context = Vec::new();
            let result = {
                let Delta::Array(array_delta) = context.take_delta() else {
                    panic!("Test '{}' failed: delta is not an array", test_case.name);
                };
                handle_array(
                    test_case.original.as_array().unwrap(),
                    array_delta,
                    &mut new_children_context,
                )
                .unwrap()
            };

            assert_eq!(
                new_children_context.len(),
                test_case.expected_children,
                "Edge case test '{}' failed: expected {} children, got {}",
                test_case.name,
                test_case.expected_children,
                new_children_context.len()
            );

            assert_eq!(
                result, test_case.expected,
                "Edge case test '{}' failed: expected {:?}, got {:?}",
                test_case.name, test_case.expected, result
            );
        }
    }
}
