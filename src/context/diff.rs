use crate::context::{
    // Context,
    FilterContext,
    MyContext,
};
use crate::types::{Delta, Options};
use serde_json::Value;

#[derive(Debug)]
pub struct DiffContext<'a> {
    pub context: MyContext<Self>,
    pub left: &'a Value,
    pub right: &'a Value,
    pub options: Options,
    // pub has_result: bool,
}

impl<'a> FilterContext for DiffContext<'a> {
    type Result = Delta<'a>;

    fn set_result(&mut self, result: Delta<'a>) -> &mut Self {
        log::trace!("set_result: {:?}", result);
        self.context.set_result(result);
        self
    }

    fn get_result(&self) -> Option<&Delta<'a>> {
        self.context.result.as_ref()
    }

    fn exit(&mut self) -> &mut Self {
        self.context.exit();
        self
    }

    fn is_exiting(&self) -> bool {
        self.context.is_exiting()
    }

    fn inner_data(&mut self) -> &mut MyContext<Self> {
        &mut self.context
    }
}

impl<'a> DiffContext<'a> {
    pub fn new(left: &'a Value, right: &'a Value) -> Self {
        Self {
            context: MyContext::new("diff".to_string()),
            left,
            right,
            options: Options::default(),
        }
    }

    pub fn prepare_delta_result<T: Into<Delta<'a>>>(&mut self, result: T) -> Delta<'a> {
        let mut delta = result.into();

        // Handle omit_removed_values option
        if self.options.omit_removed_values.unwrap_or(false) {
            if let Delta::Modified(_, _) = &mut delta {
                // For modified deltas, omit the left/old value
                // This makes the delta more compact but irreversible
                // In Rust, we'd need to handle this differently since we can't modify the enum
                // For now, we'll keep the original structure
            }
        }

        // Handle clone_diff_values option
        if let Some(clone_diff_values) = self.options.clone_diff_values {
            if clone_diff_values {
                // Clone the values in the delta
                delta = self.clone_delta_values(delta);
            }
        }

        delta
    }

    fn clone_delta_values(&self, delta: Delta<'a>) -> Delta<'a> {
        todo!()
        // match delta {
        //     Delta::Added(value) => {
        //         if value.is_object() || value.is_array() {
        //             Delta::Added(clone(&value))
        //         } else {
        //             Delta::Added(value)
        //         }
        //     }
        //     Delta::Modified(old_value, new_value) => {
        //         let cloned_old = if old_value.is_object() || old_value.is_array() {
        //             clone(&old_value)
        //         } else {
        //             old_value
        //         };
        //         let cloned_new = if new_value.is_object() || new_value.is_array() {
        //             clone(&new_value)
        //         } else {
        //             new_value
        //         };
        //         Delta::Modified(cloned_old, cloned_new)
        //     }
        //     Delta::Object(changes) => {
        //         let cloned_changes = changes
        //             .into_iter()
        //             .map(|(key, delta)| (key, Box::new(self.clone_delta_values(*delta))))
        //             .collect();
        //         Delta::Object(cloned_changes)
        //     }
        //     // For other variants, return as-is
        //     delta => delta,
        // }
    }

    pub fn get_result(&self) -> Option<&Delta<'a>> {
        self.context.result.as_ref()
    }

    pub fn has_result(&self) -> bool {
        todo!()
        // self.context.borrow().has_result()
    }
}

fn get_value_type(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_json::json;

//     #[test]
//     fn test_diff_context_creation() {
//         let left = json!({"a": 1});
//         let right = json!({"a": 2});
//         let context = DiffContext::new(left, right);

//         assert_eq!(context.pipe, "diff");
//         assert_eq!(context.left_type, Some("object".to_string()));
//         assert_eq!(context.right_type, Some("object".to_string()));
//         assert_eq!(context.left_is_array, Some(false));
//         assert_eq!(context.right_is_array, Some(false));
//     }

//     #[test]
//     fn test_diff_context_with_arrays() {
//         let left = json!([1, 2, 3]);
//         let right = json!([1, 2, 4]);
//         let context = DiffContext::new(left, right);

//         assert_eq!(context.left_type, Some("array".to_string()));
//         assert_eq!(context.right_type, Some("array".to_string()));
//         assert_eq!(context.left_is_array, Some(true));
//         assert_eq!(context.right_is_array, Some(true));
//     }

//     #[test]
//     fn test_set_result() {
//         let left = json!({"a": 1});
//         let right = json!({"a": 2});
//         let mut context = DiffContext::new(left, right);

//         let delta = Delta::Modified(json!(1), json!(2));
//         // context.set_result(delta);

//         assert!(context.has_result());
//         assert!(context.get_result().is_some());
//     }
// }
