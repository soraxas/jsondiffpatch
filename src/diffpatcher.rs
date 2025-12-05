use crate::context::{DiffContext, FilterContext, PatchContext};
use crate::pipeline::diff_pipeline::DiffPipeline;
use crate::pipeline::patch_pipeline::PatchPipeline;
use crate::processor::Processor;
use crate::types::{Delta, Options};
use serde_json::Value;

pub struct DiffPatcher {
    processor: Processor,
}

impl DiffPatcher {
    pub fn new(options: Option<Options>) -> Self {
        Self {
            processor: Processor::new(options),
        }
    }

    pub fn diff<'a>(&self, left: &'a Value, right: &'a Value) -> Option<Delta<'a>> {
        // Create a diff context

        let mut context = DiffContext::new(left, right);
        let mut diff_pipe = DiffPipeline;
        self.processor
            .process(&mut context, &mut diff_pipe)
            .expect("diff failed");

        context.get_result().cloned()
    }

    pub fn patch(&self, _left: &Value, delta: Delta) -> Option<Value> {
        // Create a patch context
        // For now, return None as the implementation is simplified
        let mut context = PatchContext::new(_left, delta);
        let mut patch_pipe = PatchPipeline;
        self.processor
            .process(&mut context, &mut patch_pipe)
            .expect("patch failed");

        context.pop_result().map(|r| r.into_owned())
    }

    pub fn reverse(&self, _delta: &Delta) -> Option<Delta<'_>> {
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
