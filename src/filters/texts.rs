use crate::context::{ContextOld, DiffContext};
use crate::processor::FilterOld;
use serde_json::Value;

pub struct TextsDiffFilter;
pub struct TextsPatchFilter;

// impl Filter<DiffContext, Delta> for TextsPatchFilter {
//     fn filter_name(&self) -> &str {
//         "texts-diff"
//     }

//     fn process(&self, context: &mut DiffContext) {
//         // This is a simplified implementation
//         // In the full implementation, this would handle trivial cases like:
//         // - Same values (no diff)
//         // - Different types
//         // - Null values
//         // - Primitive values

//         if context.left == context.right {
//             context.set_result(Delta::None).exit();
//             return;
//         }

//         if context.left.is_null() {
//             context.set_result(Delta::Added(context.right.clone())).exit();
//             return;
//         }
//         if context.right.is_null() {
//             context.set_result(Delta::Deleted(context.left.clone())).exit();
//             return;
//         }
//         if std::mem::discriminant(&context.left) != std::mem::discriminant(&context.right) {
//             context.set_result(Delta::Modified(context.left.clone(), context.right.clone())).exit();
//             return;
//         }
//         // if context.left.is_object() {
//         //     context.set_result(Delta::Object(context.left.clone())).exit();
//         // }

//         // For now, we'll just set a placeholder result
//         // context.set_result(Value::Null);
//     }
// }

impl FilterOld for TextsDiffFilter {
    fn filter_name(&self) -> &str {
        "texts-diff"
    }

    fn process(&self, context: &mut Box<dyn ContextOld>) {
        // Handle text diffing
        // This would implement:
        // - String comparison
        // - Text diff algorithms
        // - Character-level changes

        let context: DiffContext = todo!();

        // let context = context.as_any().downcast_mut::<DiffContext>().unwrap();

        if !context.left.is_string() {
            return;
        }

        let left = context.left.as_str().unwrap();
        let right = context.right.as_str().unwrap();
        let min_length = context
            .options
            .text_diff
            .as_ref()
            .unwrap()
            .min_length
            .unwrap_or(10);

        if left.len() < min_length || right.len() < min_length {
            // context.set_result(vec![left, right]);
        }

        // // return if right is not a string or not a some value
        // if !context.right.is_string() {
        //     return;
        // }
        // let left = context.left.as_str().unwrap();
        // let right = context.right.as_str().unwrap();

        // context.set_result(Value::Null);
    }
}

impl FilterOld for TextsPatchFilter {
    fn filter_name(&self) -> &str {
        "texts-patch"
    }

    fn process(&self, context: &mut Box<dyn ContextOld>) {
        // Handle text patching
        // This would apply text deltas to reconstruct the target text

        context.set_result(Value::Null);
    }
}

pub struct TextsReverseFilter;

impl FilterOld for TextsReverseFilter {
    fn filter_name(&self) -> &str {
        "texts-reverse"
    }

    fn process(&self, context: &mut Box<dyn ContextOld>) {
        // Handle text reverse operations
        // This would reverse text delta operations

        context.set_result(Value::Null);
    }
}

pub fn create_texts_filters() -> Vec<Box<dyn FilterOld>> {
    vec![
        Box::new(TextsDiffFilter),
        Box::new(TextsPatchFilter),
        Box::new(TextsReverseFilter),
    ]
}
