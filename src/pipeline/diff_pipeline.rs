use crate::context::{DiffContext, FilterContext};
use crate::errors::JsonDiffPatchError;
use crate::pipeline::arrays::{post_process_arrays_diff, process_arrays_diff};
use crate::pipeline::texts::process_text_diff;
use crate::processor::Pipeline;
use crate::types::Delta;
use serde_json::Value;
use std::collections::HashMap;

pub struct DiffPipeline;

impl<'a> Pipeline<DiffContext<'a>, Delta<'a>> for DiffPipeline {
    fn filter_name(&self) -> &str {
        "diff-pipeline"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) -> Result<(), JsonDiffPatchError> {
        if context.left == context.right {
            // same value
            context.set_result(Delta::None).exit();
        } else if context.right.is_null() {
            // new added value
            context.set_result(Delta::deleted_ref(context.left)).exit();
        } else if context.left.is_boolean()
            || context.left.is_number()
            || (std::mem::discriminant(context.left) != std::mem::discriminant(context.right))
        {
            // trivial value / different types
            context
                .set_result(Delta::modified_ref(context.left, context.right))
                .exit();
        } else {
            // now left's type must equals to right's type
            match &context.left {
                Value::Object(object) => {
                    let left = object;
                    let right = context.right.as_object().expect("right is an object");
                    // TODO: add property filter

                    // process keys from left (with potential deletions)
                    for (key, value) in left {
                        new_children_context.push((
                            key.to_string(),
                            DiffContext::new(value, right.get(key).unwrap_or(&Value::Null)),
                        ));
                    }

                    // process keys from right (with potential additions)
                    for (key, value) in right {
                        if !left.contains_key(key) {
                            new_children_context
                                .push((key.to_string(), DiffContext::new(&Value::Null, value)));
                        }
                    }

                    if new_children_context.is_empty() {
                        context.set_result(Delta::None).exit();
                        return Ok(());
                    }

                    context.exit();
                }
                Value::Array(array) => {
                    process_arrays_diff(
                        context,
                        array,
                        context.right.as_array().expect("right is an array"),
                        new_children_context,
                    )?;
                }
                Value::Null => {
                    context.set_result(Delta::added_ref(context.right)).exit();
                }
                Value::String(string) => {
                    process_text_diff(
                        context,
                        string,
                        context.right.as_str().expect("right is a string"),
                    )?;
                }
                Value::Bool(_) => unreachable!(),
                Value::Number(_) => unreachable!(),
            }
        }
        Ok(())
    }

    fn post_process(
        &self,
        context: &mut DiffContext<'a>,
        children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) -> Result<(), JsonDiffPatchError> {
        match &context.left {
            Value::Object(_object) => {
                let mut result = HashMap::new();

                for (key, child) in children_context {
                    if let Some(child_result) = child.pop_result() {
                        result.insert(key.clone(), child_result);
                    }
                }
                if result.is_empty() {
                    return Ok(());
                }

                context.set_result(Delta::Object(result)).exit();
            }
            Value::Array(_array) => {
                post_process_arrays_diff(context, children_context)?;
                return Ok(());
            }
            Value::Bool(_) => {}
            Value::Null => {}
            Value::Number(_) => {}
            Value::String(_) => {}
        }
        Ok(())
    }
}
