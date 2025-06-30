use crate::context::{ContextData, FilterContext};
use crate::types::Delta;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug)]
pub struct PatchContext<'a> {
    context_data: ContextData<Self>,
    pub left: &'a Value,
    delta: DeltaWithLeftover<'a>,
}

/// A delta wrapper that contains a leftover indicator of its original delta
#[derive(Debug, Copy, Clone)]
pub enum DeltaIndicator {
    Object,
    Array,
    TextDiff,
    Added,
    Modified,
    Deleted,
    Moved,
    None,
}

impl DeltaIndicator {
    pub fn new_from_delta<'a>(delta: &Delta<'a>) -> Self {
        match delta {
            Delta::Object(_) => DeltaIndicator::Object,
            Delta::Array(_) => DeltaIndicator::Array,
            Delta::TextDiff(_) => DeltaIndicator::TextDiff,
            Delta::Added(_) => DeltaIndicator::Added,
            Delta::Modified(_, _) => DeltaIndicator::Modified,
            Delta::Deleted(_) => DeltaIndicator::Deleted,
            Delta::Moved { .. } => DeltaIndicator::Moved,
            Delta::None => DeltaIndicator::None,
        }
    }
}

#[derive(Debug)]
pub enum DeltaWithLeftover<'a> {
    Delta(Delta<'a>),
    Leftover(DeltaIndicator),
}

impl<'a> DeltaWithLeftover<'a> {
    pub fn peek(&self) -> DeltaIndicator {
        match self {
            DeltaWithLeftover::Delta(delta) => DeltaIndicator::new_from_delta(delta),
            DeltaWithLeftover::Leftover(indicator) => *indicator,
        }
    }
}

impl<'a> PatchContext<'a> {
    pub fn new(left: &'a Value, delta: Delta<'a>) -> Self {
        Self {
            left,
            delta: DeltaWithLeftover::Delta(delta),
            context_data: ContextData::new(),
        }
    }

    pub fn take_delta(&mut self) -> Delta<'a> {
        let replacement = match &mut self.delta {
            DeltaWithLeftover::Delta(delta) => {
                DeltaWithLeftover::Leftover(DeltaIndicator::new_from_delta(delta))
            }
            DeltaWithLeftover::Leftover(_leftover) => {
                panic!("delta was already taken")
            }
        };

        let prev_delta = std::mem::replace(&mut self.delta, replacement);

        match prev_delta {
            DeltaWithLeftover::Delta(delta) => delta,
            DeltaWithLeftover::Leftover(_) => {
                panic!("delta was already taken")
            }
        }
    }

    pub fn peek_delta(&self) -> DeltaIndicator {
        self.delta.peek()
    }
}

impl<'a> FilterContext for PatchContext<'a> {
    type Result = Cow<'a, Value>;

    fn inner_data(&self) -> &ContextData<Self> {
        &self.context_data
    }

    fn inner_data_mut(&mut self) -> &mut ContextData<Self> {
        &mut self.context_data
    }
}
