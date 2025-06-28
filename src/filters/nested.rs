use crate::context::{DiffContext, FilterContext};
use crate::processor::Filter;
use crate::types::Delta;
use serde_json::Value;
use std::collections::HashMap;

pub struct CollectionChildrenDiffFilter;
pub struct CollectionChildrenPatchFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for CollectionChildrenDiffFilter {
    fn filter_name(&self) -> &str {
        "collection-children-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext,
        new_children_context: &mut Vec<(String, DiffContext)>,
    ) {
    }

    fn post_process(
        &self,
        context: &mut DiffContext<'a>,
        children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // let mut context_mut = context.borrow_mut();
        // This is a simplified implementation
        // In the full implementation, this would handle trivial cases like:
        // - Same values (no diff)
        // - Different types
        // - Null values
        // - Primitive values

        let result: Delta<'a> = if context.left.is_object() {
            let mut result = HashMap::new();

            for (key, child) in children_context {
                if let Some(child_result) = child.get_result() {
                    result.insert(key.clone(), child_result.clone());
                }
            }
            if result.is_empty() {
                return;
            }
            Delta::Object(result)
        } else if context.left.is_array() {
            return;

            let result = Vec::new();

            for (key, child) in children_context {
                if let Some(child_result) = child.get_result() {
                    // TODO:
                    // result.push((key.clone(), child_result.clone()));
                }
            }
            if result.is_empty() {
                return;
            }
            Delta::Array(result)
        } else {
            return;
        };

        context.set_result(result).exit();
    }
}

pub struct ObjectsDiffFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for ObjectsDiffFilter {
    fn filter_name(&self) -> &str {
        "objects-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // let mut context_mut = context.borrow_mut();
        if !context.left.is_object() {
            return;
        }

        let left = context.left.as_object().unwrap();
        let right = context.right.as_object().unwrap();

        // TODO: add property filter

        for (key, value) in left {
            new_children_context.push((
                key.to_string(),
                DiffContext::new(value, right.get(key).unwrap_or(&Value::Null)),
            ));
        }

        for (key, value) in right {
            if !left.contains_key(key) {
                new_children_context.push((key.to_string(), DiffContext::new(&Value::Null, value)));
            }
        }

        if new_children_context.is_empty() {
            context.set_result(Delta::None).exit();
            return;
        }

        context.exit();
    }

    fn post_process(
        &self,
        context: &mut DiffContext,
        new_children_context: &mut Vec<(String, DiffContext)>,
    ) {
    }
}
