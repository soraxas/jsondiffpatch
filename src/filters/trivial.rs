use crate::context::{ContextOld, DiffContext, FilterContext};
use crate::processor::{Filter, FilterOld};
use crate::types::Delta;

pub struct TrivialDiffFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for TrivialDiffFilter {
    fn filter_name(&self) -> &str {
        "trivial-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // This is a simplified implementation
        // In the full implementation, this would handle trivial cases like:
        // - Same values (no diff)
        // - Different types
        // - Null values
        // - Primitive values

        // let mut context_mut = context.borrow_mut();
        let left = context.left;
        let right = context.right;

        if left == right {
            context.set_result(Delta::None).exit();
        } else if left.is_null() {
            context.set_result(Delta::Added(right)).exit();
        } else if right.is_null() {
            context.set_result(Delta::Deleted(left)).exit();
        } else if left.is_boolean()
            || left.is_number()
            || (std::mem::discriminant(left) != std::mem::discriminant(right))
        {
            context.set_result(Delta::Modified(left, right)).exit();
        }

        // if context.left.is_object() {
        //     context.set_result(Delta::Object(context.left.clone())).exit();
        // }

        // For now, we'll just set a placeholder result
        // context.set_result(Value::Null);
    }

    fn post_process(
        &self,
        context: &mut DiffContext<'a>,
        new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // Handle trivial post-process operations
        // This would handle cases like:
        // - Added deltas
        // - Modified deltas
        // - Deleted deltas
    }
}

pub struct TrivialPatchFilter;

impl FilterOld for TrivialPatchFilter {
    fn filter_name(&self) -> &str {
        "trivial-patch"
    }

    fn process(&self, context: &mut Box<dyn ContextOld>) {
        // Handle trivial patch operations
        // This would handle cases like:
        // - Added deltas
        // - Modified deltas
        // - Deleted deltas

        // context.set_result(Value::Null).exit();
    }
}

pub struct TrivialReverseFilter;
