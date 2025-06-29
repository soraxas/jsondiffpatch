use crate::context::{DiffContext, FilterContext};
use crate::processor::Filter;
use crate::types::Delta;

pub struct TrivialDiffFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for TrivialDiffFilter {
    fn filter_name(&self) -> &str {
        "trivial-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        _new_children_context: &mut Vec<(String, DiffContext<'a>)>,
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
    }
}
