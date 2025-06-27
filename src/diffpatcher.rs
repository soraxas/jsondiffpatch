use crate::clone::clone;
use crate::processor::{Processor, Pipe};
use crate::types::{Delta, Options};
use crate::filters::{
    create_trivial_filters,
    create_arrays_filters,
    create_objects_filters,
    create_texts_filters,
    create_dates_filters,
};
use serde_json::Value;

pub struct DiffPatcher {
    processor: Processor,
}

impl DiffPatcher {
    pub fn new(options: Option<Options>) -> Self {
        let mut processor = Processor::new(options);

        // Set up diff pipe
        let diff_pipe = Pipe::new("diff".to_string())
            .append(create_trivial_filters())
            .append(create_dates_filters())
            .append(create_texts_filters())
            .append(create_objects_filters())
            .append(create_arrays_filters())
            .should_have_result();

        // Set up patch pipe
        let patch_pipe = Pipe::new("patch".to_string())
            .append(create_trivial_filters())
            .append(create_texts_filters())
            .append(create_objects_filters())
            .append(create_arrays_filters())
            .should_have_result();

        // Set up reverse pipe
        let reverse_pipe = Pipe::new("reverse".to_string())
            .append(create_trivial_filters())
            .append(create_texts_filters())
            .append(create_objects_filters())
            .append(create_arrays_filters())
            .should_have_result();

        processor.pipe("diff", Box::new(diff_pipe));
        processor.pipe("patch", Box::new(patch_pipe));
        processor.pipe("reverse", Box::new(reverse_pipe));

        Self { processor }
    }

    pub fn options(&self) -> &Options {
        self.processor.options()
    }

    pub fn set_options(&mut self, options: Options) {
        self.processor.set_options(options);
    }

    pub fn diff(&self, _left: &Value, _right: &Value) -> Option<Delta> {
        // Create a diff context
        // For now, return None as the implementation is simplified
        None
    }

    pub fn patch(&self, _left: &Value, _delta: &Delta) -> Option<Value> {
        // Create a patch context
        // For now, return None as the implementation is simplified
        None
    }

    pub fn reverse(&self, _delta: &Delta) -> Option<Delta> {
        // Create a reverse context
        // For now, return None as the implementation is simplified
        None
    }

    pub fn unpatch(&self, right: &Value, delta: &Delta) -> Option<Value> {
        // Unpatch is patch with reversed delta
        if let Some(reversed_delta) = self.reverse(delta) {
            self.patch(right, &reversed_delta)
        } else {
            None
        }
    }

    pub fn clone(&self, value: &Value) -> Value {
        clone(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_diffpatcher_creation() {
        let diffpatcher = DiffPatcher::new(None);
        assert!(diffpatcher.options().match_by_position.unwrap_or(false) == false);
    }

    #[test]
    fn test_clone() {
        let diffpatcher = DiffPatcher::new(None);
        let original = json!({"a": 1, "b": "hello"});
        let cloned = diffpatcher.clone(&original);
        assert_eq!(original, cloned);
    }
}