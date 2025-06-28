pub mod diff;

pub use diff::DiffContext;

use crate::types::Options;
use serde_json::Value;

pub trait FilterContext {
    type Result;

    fn set_result(&mut self, result: Self::Result) -> &mut Self;
    fn get_result(&self) -> Option<&Self::Result>;
    fn exit(&mut self) -> &mut Self;
    fn is_exiting(&self) -> bool;

    fn inner_data(&mut self) -> &mut ContextData<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct ContextData<FC: FilterContext> {
    result: Option<FC::Result>,
    exiting: bool,
    pub options: Option<Options>,
}

impl<FC: FilterContext> ContextData<FC> {
    pub fn new() -> Self {
        Self {
            result: None,
            exiting: false,
            options: None,
        }
    }

    pub fn is_exiting(&self) -> bool {
        self.exiting
    }

    pub fn set_result(&mut self, result: FC::Result) {
        self.result = Some(result);
    }

    pub fn exit(&mut self) {
        self.exiting = true;
    }
}
