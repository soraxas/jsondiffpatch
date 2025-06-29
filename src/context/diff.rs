use crate::context::{ContextData, FilterContext};
use crate::types::Delta;
use serde_json::Value;

#[derive(Debug)]
pub struct DiffContext<'a> {
    context_data: ContextData<Self>,
    pub left: &'a Value,
    pub right: &'a Value,
}

impl<'a> FilterContext for DiffContext<'a> {
    type Result = Delta<'a>;

    fn skip_set_result_filter(&mut self, result: &Self::Result) -> bool {
        // if the result is None, skip setting the result
        matches!(result, Delta::None)
    }

    fn inner_data(&self) -> &ContextData<Self> {
        &self.context_data
    }

    fn inner_data_mut(&mut self) -> &mut ContextData<Self> {
        &mut self.context_data
    }
}

impl<'a> DiffContext<'a> {
    pub fn new(left: &'a Value, right: &'a Value) -> Self {
        Self {
            left,
            right,
            context_data: ContextData::new(),
        }
    }
}
