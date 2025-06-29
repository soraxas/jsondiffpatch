use crate::context::{ContextData, FilterContext};
use crate::types::{Delta, Options};
use serde_json::Value;
use std::rc::Rc;

#[derive(Debug)]
pub struct ReverseContext<'a> {
    context_data: ContextData<Self>,
    pub delta: Delta<'a>,
    pub new_name: Option<&'a str>,
}

impl<'a> FilterContext for ReverseContext<'a> {
    type Result = Delta<'a>;

    fn inner_data(&self) -> &ContextData<Self> {
        &self.context_data
    }

    fn inner_data_mut(&mut self) -> &mut ContextData<Self> {
        &mut self.context_data
    }
}

impl<'a> ReverseContext<'a> {
    pub fn new(delta: Delta<'a>, options: Rc<Options>) -> Self {
        Self {
            delta,
            context_data: ContextData::new(options),
            new_name: None,
        }
    }
}
