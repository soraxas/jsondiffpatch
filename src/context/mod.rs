pub mod diff;
pub mod patch;
pub mod reverse;

pub use diff::DiffContext;
pub use patch::PatchContext;

use crate::types::Options;
use std::rc::Rc;

/// A trait that defines the interface for filter contexts.
///
/// Filter contexts are used to store the result of a filter and to track the state of the filter.
pub trait FilterContext
where
    Self: Sized,
{
    type Result;

    /// a filter that can be used to skip setting the result
    fn skip_set_result_filter(&mut self, _result: &Self::Result) -> bool {
        false
    }

    fn set_result(&mut self, result: Self::Result) -> &mut Self
    where
        Self::Result: std::fmt::Debug,
    {
        if self.skip_set_result_filter(&result) {
            return self;
        }
        log::trace!("set_result: {:?}", result);
        self.inner_data_mut().set_result(result);
        self
    }

    fn get_result(&self) -> Option<&Self::Result> {
        self.inner_data().result.as_ref()
    }

    fn get_result_mut(&mut self) -> Option<&mut Self::Result> {
        self.inner_data_mut().result.as_mut()
    }

    fn exit(&mut self) -> &mut Self {
        self.inner_data_mut().exit();
        self
    }

    fn is_exiting(&self) -> bool {
        self.inner_data().is_exiting()
    }

    fn options(&self) -> &Rc<Options> {
        &self.inner_data().options
    }

    fn inner_data(&self) -> &ContextData<Self>;
    fn inner_data_mut(&mut self) -> &mut ContextData<Self>;
}

#[derive(Debug, Default)]
pub struct ContextData<FC: FilterContext> {
    result: Option<FC::Result>,
    exiting: bool,
    pub options: Rc<Options>,
}

impl<FC: FilterContext> ContextData<FC> {
    pub fn new(options: Rc<Options>) -> Self {
        Self {
            result: None,
            exiting: false,
            options,
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
