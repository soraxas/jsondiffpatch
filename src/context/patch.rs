use crate::context::{ContextData, FilterContext};
use crate::types::Delta;
use serde_json::Value;

#[derive(Debug)]
pub struct PatchContext<'a> {
    context_data: ContextData<Self>,
    pub left: &'a Value,
    pub delta: Delta<'a>,
}

impl<'a> FilterContext for PatchContext<'a> {
    type Result = Value;

    fn inner_data(&self) -> &ContextData<Self> {
        &self.context_data
    }

    fn inner_data_mut(&mut self) -> &mut ContextData<Self> {
        &mut self.context_data
    }
}

impl<'a> PatchContext<'a> {
    pub fn new(left: &'a Value, delta: Delta<'a>) -> Self {
        Self {
            left,
            delta,
            context_data: ContextData::new(),
        }
    }
}
