use crate::context::FilterContext;
use crate::errors::JsonDiffPatchError;
use crate::types::Options;
use crate::types::OPTIONS;

pub trait Pipeline<C, TResult> {
    fn filter_name(&self) -> &str;
    fn process(
        &self,
        _context: &mut C,
        _new_children_context: &mut Vec<(String, C)>,
    ) -> Result<(), JsonDiffPatchError> {
        Ok(())
    }
    fn post_process(
        &self,
        _context: &mut C,
        _new_children_context: &mut Vec<(String, C)>,
    ) -> Result<(), JsonDiffPatchError> {
        Ok(())
    }
}

pub struct Processor {}

impl Processor {
    pub fn new(options: Option<Options>) -> Self {
        OPTIONS
            .set(options.unwrap_or_default())
            .expect("options failed to set");
        Self {}
    }

    pub fn process<TContext: FilterContext>(
        &self,
        context: &mut TContext,
        pipeline: &mut impl Pipeline<TContext, TContext::Result>,
    ) -> Result<(), JsonDiffPatchError> {
        process_pipeline(context, pipeline)
    }
}

fn process_pipeline<TContext: FilterContext>(
    context: &mut TContext,
    pipeline: &mut impl Pipeline<TContext, TContext::Result>,
) -> Result<(), JsonDiffPatchError> {
    let mut new_children_context = vec![];

    pipeline.process(context, &mut new_children_context)?;

    if new_children_context.is_empty() {
        // continue to process the next queue
        // continue;
    }

    for (_key, child) in &mut new_children_context {
        // recursively process the child context
        // it's better to use a stack here, as, if the json is too deep, it will cause a stack overflow
        process_pipeline(child, pipeline)?;
    }

    pipeline.post_process(context, &mut new_children_context)?;

    Ok(())
}
