use crate::context::{DiffContext, FilterContext, PatchContext};
use crate::filters::diff_pipeline::DiffPipeline;
use crate::filters::patch_pipeline::PatchPipeline;
use crate::processor::{Pipe, Processor};
use crate::types::{Delta, Options};
use serde_json::Value;
use std::rc::Rc;

pub fn build_diff_pipe<'a>() -> Pipe<DiffContext<'a>, Delta<'a>> {
    Pipe::new("diff".to_string())
        .append(Box::new(DiffPipeline))
        .should_have_result()
}

pub fn build_patch_pipe<'a>() -> Pipe<PatchContext<'a>, Value> {
    Pipe::new("patch".to_string())
        .append(Box::new(PatchPipeline))
        .should_have_result()
}

pub struct DiffPatcher {
    // processor: Processor,
}

impl DiffPatcher {
    pub fn new(options: Option<Options>) -> Self {
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
            // diff_pipe: diff_pipe,
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

    pub fn diff<'a>(&self, left: &'a Value, right: &'a Value) -> Option<Delta<'a>> {
        // Create a diff context

        let options = Rc::new(Options::default());

        let mut context = DiffContext::new(left, right, options);
        let mut diff_pipe = build_diff_pipe();

        let processor = Processor::new(None);
        processor.process(&mut context, &mut diff_pipe);

        context.get_result().cloned()
    }

    pub fn patch(&self, _left: &Value, delta: Delta) -> Option<Value> {
        // Create a patch context
        // For now, return None as the implementation is simplified
        let options = Rc::new(Options::default());
        let mut context = PatchContext::new(_left, delta, options);
        let mut patch_pipe = build_patch_pipe();
        let processor = Processor::new(None);
        processor.process(&mut context, &mut patch_pipe);
        context.get_result().cloned()
    }

    pub fn reverse(&self, _delta: &Delta) -> Option<Delta> {
        // Create a reverse context
        // For now, return None as the implementation is simplified
        None
    }

    pub fn unpatch(&self, right: &Value, delta: &Delta) -> Option<Value> {
        // Unpatch is patch with reversed delta
        if let Some(reversed_delta) = self.reverse(delta) {
            self.patch(right, reversed_delta)
        } else {
            None
        }
    }
}
