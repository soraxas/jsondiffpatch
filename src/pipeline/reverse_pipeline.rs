use crate::context::patch::DeltaIndicator;
use crate::context::{FilterContext, PatchContext};
use crate::errors::JsonDiffPatchError;
use crate::pipeline::texts::DMP;
use crate::processor::Pipeline;
use crate::types::{ArrayDeltaIndex, Delta};
use diff_match_patch_rs::Efficient;
use serde_json::{Map, Value};
use std::borrow::Cow;

pub struct ReversePipeline;

impl<'a> Pipeline<PatchContext<'a>, Value> for ReversePipeline {
    fn filter_name(&self) -> &str {
        "reverse-pipeline"
    }

    fn process(
        &self,
        context: &mut PatchContext<'a>,
        new_children_context: &mut Vec<(String, PatchContext<'a>)>,
    ) -> Result<(), JsonDiffPatchError> {
        match context.take_delta() {
            Delta::Object(_object_delta) => {
                todo!()
            }
            Delta::Array(array_delta) => {
                let mut container = vec![];
                let result = handle_array(
                    context.left.as_array().ok_or_else(|| {
                        JsonDiffPatchError::InvalidPatchToTarget {
                            patch: "array".to_string(),
                        }
                    })?,
                    &array_delta,
                    &mut container,
                )?;
                // handle new children
                for (name, value, delta) in container {
                    let child_context = PatchContext::new(value, delta);
                    new_children_context.push((name, child_context));
                }

                context.set_result(Cow::Owned(result)).exit();
            }
            Delta::Added(new_value) => {
                context.set_result(new_value);
            }
            Delta::Deleted(_old_value) => {
                // dont apply this value to the result to keep it as deleted
            }
            Delta::Modified(_from, to) => {
                context.set_result(to);
            }
            Delta::Moved {
                new_index: _,
                moved_value: _,
            } => {
                return Err(JsonDiffPatchError::InternalPatchLogicError(
                    "Should be handled by array directly, as move does not make sense for non-array container".to_string(),
                ));
            }
            Delta::TextDiff(text_diff) => {
                let Value::String(left_txt) = context.left else {
                    return Err(JsonDiffPatchError::InvalidPatchToTarget {
                        patch: "text diff".to_string(),
                    });
                };
                let left_txt = left_txt.as_str();
                // context.set_result(text_diff.clone()).exit();
                match DMP.patch_from_text::<Efficient>(text_diff.as_str()) {
                    Ok(patches) => {
                        let (new_txt, ops) = DMP.patch_apply(&patches, left_txt)?;
                        ops.iter().for_each(|op| {
                            if !op {
                                log::error!("some text-diff patch applied failed");
                            }
                        });

                        context
                            .set_result(Cow::Owned(Value::String(new_txt)))
                            .exit();
                    }
                    Err(e) => {
                        return Err(JsonDiffPatchError::ApplyTextDiffFailed(e));
                    }
                }
            }
            Delta::None => {}
        }
        Ok(())
    }

    fn post_process(
        &self,
        context: &mut PatchContext<'a>,
        children_context: &mut Vec<(String, PatchContext<'a>)>,
    ) -> Result<(), JsonDiffPatchError> {
        match context.peek_delta() {
            DeltaIndicator::Array => {
                // Collect results from children and apply them to the array

                let current_result = context
                    .get_result_mut()
                    .expect("should be set during the main patch process");
                let array_mut = current_result.to_mut().as_array_mut().ok_or_else(|| {
                    JsonDiffPatchError::InvalidPatchToTarget {
                        patch: "array".to_string(),
                    }
                })?;

                for (index_str, child_context) in children_context {
                    if let Some(child_result) = child_context.pop_result() {
                        if let Ok(index) = index_str.parse::<usize>() {
                            if index < array_mut.len() {
                                array_mut[index] = child_result.into_owned();
                            }
                        }
                    }
                }
                // array is modified in-place.
                // context.set_result(Value::Array(array)).exit();
            }
            DeltaIndicator::Object => {
                let mut reversed_changes = Map::new();

                // Collect results from children and apply them to the object
                for (key, child_context) in children_context
                    .iter_mut()
                    .filter_map(|(key, context)| context.pop_result().map(|result| (key, result)))
                {
                    reversed_changes.insert(key.clone(), child_context.into_owned());
                }
                context
                    .set_result(Cow::Owned(Value::Object(reversed_changes)))
                    .exit();
            }
            _ => {}
        };
        Ok(())
    }
}

pub(crate) fn handle_array<'a>(
    left: &'a [Value],
    array_delta: &Vec<(ArrayDeltaIndex, Delta<'a>)>,
    return_container: &mut Vec<(String, &'a Value, Delta<'a>)>,
) -> Result<Value, JsonDiffPatchError> {
    let mut new_array = left.to_vec();

    let mut to_insert: Vec<(usize, Value)> = Vec::new();

    // create a vector of references to the array delta, but with the indices sorted
    let mut sorted_array_delta = array_delta
        .iter()
        .map(|(index, delta)| (index, delta))
        .collect::<Vec<_>>();

    // Remove items, in reverse order to avoid index shifting issues
    sorted_array_delta.sort_by_key(|(index, _)| *index);

    for (index, delta) in sorted_array_delta.iter().rev() {
        match index {
            ArrayDeltaIndex::RemovedOrMoved(removed_index) => {
                // Check if it's a removal or move
                if *removed_index >= new_array.len() {
                    return Err(JsonDiffPatchError::IndexOutOfBoundsRemove {
                        index: *removed_index,
                        length: new_array.len(),
                    });
                }

                // Check if this was a move operation
                let removed_value = new_array.remove(*removed_index);
                match delta {
                    Delta::Deleted(_) => {
                        // to_remove.push((*removed_index, None));
                    }
                    Delta::Moved { new_index, .. } => {
                        // We'll handle the reinsertion later, as we want to insert in increasing order
                        to_insert.push((*new_index, removed_value));
                    }
                    _ => {
                        return Err(JsonDiffPatchError::InvalidPatch(
                            "only removal or move can be applied at original array indices"
                                .to_string(),
                        ));
                    }
                }
            }
            ArrayDeltaIndex::NewOrModified(new_index) => {
                match delta {
                    Delta::Added(value) => {
                        to_insert.push((*new_index, (*value.as_ref()).clone()));
                    }
                    Delta::Modified(_from, _to) => {
                        // Modified item - will be handled by child contexts
                        let value = &left[*new_index];
                        return_container.push((new_index.to_string(), value, (*delta).clone()));
                    }
                    _ => {
                        return Err(JsonDiffPatchError::InvalidPatch(
                            "only addition or modification can be applied at new array indices"
                                .to_string(),
                        ));
                    }
                }
            }
        }
    }

    // Insert items, sorted by index
    to_insert.sort_by_key(|(index, _)| *index);
    for (index, value) in to_insert {
        if index > new_array.len() {
            return Err(JsonDiffPatchError::IndexOutOfBoundsInsert {
                index,
                length: new_array.len(),
            });
        }
        new_array.insert(index, value);
    }

    Ok(Value::Array(new_array))
}
