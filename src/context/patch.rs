use crate::context::{ContextData, FilterContext};
use crate::types::{Delta, Options};
use serde_json::Value;
use std::rc::Rc;

#[derive(Debug)]
pub struct PatchContext<'a> {
    context_data: ContextData<Self>,
    pub left: &'a Value,
    pub delta: Delta<'a>,
    pub nested: bool,
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
    pub fn new(left: &'a Value, delta: Delta<'a>, options: Rc<Options>) -> Self {
        Self {
            left,
            delta,
            nested: false,
            context_data: ContextData::new(options),
        }
    }
}
