use crate::context::{DiffContext, FilterContext};
use crate::lcs;
use crate::processor::Filter;
use crate::types::{ArrayDeltaIndex, ArrayOptions, Delta};

pub struct ArraysDiffFilter;

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

pub struct ArraysPatchFilter;
