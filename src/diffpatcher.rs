use crate::clone::clone;
use crate::context::DiffContext;
use crate::filters::arrays::ArraysDiffFilter;
use crate::filters::nested::{CollectionChildrenDiffFilter, ObjectsDiffFilter};
use crate::filters::texts::TextsDiffFilter;
use crate::filters::TrivialDiffFilter;
use crate::processor::{Pipe, Processor};
use crate::types::{Delta, Options};
use serde_json::Value;

pub struct DiffPatcher {
    // processor: Processor,
}

impl DiffPatcher {
    pub fn new(options: Option<Options>) -> Self {
        // let mut processor = Processor::new(options);

        // // Set up diff pipe
        // let diff_pipe = Pipe::new("diff".to_string())
        //     .append(create_trivial_filters())
        //     .append(create_dates_filters())
        //     .append(create_texts_filters())
        //     .append(create_objects_filters())
        //     .append(create_arrays_filters())
        //     .should_have_result();

        // // Set up patch pipe
        // let patch_pipe = Pipe::new("patch".to_string())
        //     .append(create_trivial_filters())
        //     .append(create_texts_filters())
        //     .append(create_objects_filters())
        //     .append(create_arrays_filters())
        //     .should_have_result();

        // // Set up reverse pipe
        // let reverse_pipe = Pipe::new("reverse".to_string())
        //     .append(create_trivial_filters())
        //     .append(create_texts_filters())
        //     .append(create_objects_filters())
        //     .append(create_arrays_filters())
        //     .should_have_result();

        // processor.pipe("diff", Box::new(diff_pipe));
        // processor.pipe("patch", Box::new(patch_pipe));
        // processor.pipe("reverse", Box::new(reverse_pipe));

        Self {
            // processor
        }
    }

    pub fn options(&self) -> &Options {
        todo!()
        // self.processor.options()
    }

    pub fn set_options(&mut self, options: Options) {
        todo!()
        // self.processor.set_options(options);
    }

    pub fn diff(&self, left: &Value, right: &Value) -> Option<Delta> {
        // Create a diff context
        let mut context = DiffContext::new(left, right);

        // panic!("test");

        let mut diff_pipe = Pipe::new("diff".to_string())
            // .append(Box::new(CollectionChildrenDiffFilter))
            .append(Box::new(CollectionChildrenDiffFilter))
            .append(Box::new(TrivialDiffFilter))
            .append(Box::new(TextsDiffFilter))
            .append(Box::new(ObjectsDiffFilter))
            .append(Box::new(ArraysDiffFilter))
            // .append(Box::new(CollectionChildrenReverseFilter))
            // .append(create_dates_filters())
            // .append(create_texts_filters())
            // .append(create_objects_filters())
            // .append(create_arrays_filters())
            .should_have_result();

        // diff_pipe.process(&mut context);

        let processor = Processor::new(None);
        processor.process(&mut context, &mut diff_pipe);

        let res = context.get_result().unwrap().clone();
        dbg!(&res);

        println!(">> {}", &res.to_serializable().to_string());

        // For now, return None as the implementation is simplified
        // In a full implementation, this would process the context through the pipeline
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
        assert!(!diffpatcher.options().match_by_position.unwrap_or(false));
    }

    #[test]
    fn test_clone() {
        let diffpatcher = DiffPatcher::new(None);
        let original = json!({"a": 1, "b": "hello"});
        let cloned = diffpatcher.clone(&original);
        assert_eq!(original, cloned);
    }
}
