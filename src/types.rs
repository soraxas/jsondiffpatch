use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct Options {
    pub object_hash: Option<Box<dyn Fn(&Value, Option<usize>) -> Option<String> + Send + Sync>>,
    pub match_by_position: Option<bool>,
    pub arrays: Option<ArrayOptions>,
    pub text_diff: Option<TextDiffOptions>,
    pub property_filter: Option<Box<dyn Fn(&str, &DiffContext) -> bool + Send + Sync>>,
    pub clone_diff_values: Option<bool>,
    pub omit_removed_values: Option<bool>,
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("object_hash", &if self.object_hash.is_some() { "Some(Fn)" } else { "None" })
            .field("match_by_position", &self.match_by_position)
            .field("arrays", &self.arrays)
            .field("text_diff", &self.text_diff)
            .field("property_filter", &if self.property_filter.is_some() { "Some(Fn)" } else { "None" })
            .field("clone_diff_values", &self.clone_diff_values)
            .field("omit_removed_values", &self.omit_removed_values)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayOptions {
    pub detect_move: Option<bool>,
    pub include_value_on_move: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDiffOptions {
    pub min_length: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            object_hash: None,
            match_by_position: Some(false),
            arrays: Some(ArrayOptions {
                detect_move: Some(true),
                include_value_on_move: Some(false),
            }),
            text_diff: Some(TextDiffOptions {
                min_length: Some(60),
            }),
            property_filter: None,
            clone_diff_values: Some(false),
            omit_removed_values: Some(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Delta {
    Added(Value),
    Modified(Value, Value),
    Deleted(Value, i32, i32), // [value, 0, 0]
    Object(HashMap<String, Box<Delta>>),
    Array(ArrayDelta),
    Moved(Value, i32, i32), // [value, index, 3]
    TextDiff(String, i32, i32), // [text, 0, 2]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayDelta {
    #[serde(rename = "_t")]
    pub type_marker: String,
    #[serde(flatten)]
    pub changes: HashMap<String, Box<Delta>>,
}

impl ArrayDelta {
    pub fn new() -> Self {
        Self {
            type_marker: "a".to_string(),
            changes: HashMap::new(),
        }
    }
}

// Type guards (similar to TypeScript functions)
pub fn is_added_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Added(_))
}

pub fn is_modified_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Modified(_, _))
}

pub fn is_deleted_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Deleted(_, 0, 0))
}

pub fn is_object_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Object(_))
}

pub fn is_array_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Array(_))
}

pub fn is_moved_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::Moved(_, _, 3))
}

pub fn is_text_diff_delta(delta: &Delta) -> bool {
    matches!(delta, Delta::TextDiff(_, 0, 2))
}

// Context types for internal use
#[derive(Debug)]
pub struct DiffContext {
    pub left: Value,
    pub right: Value,
    pub options: Options,
}

impl DiffContext {
    pub fn new(left: Value, right: Value) -> Self {
        Self {
            left,
            right,
            options: Options::default(),
        }
    }
}

#[derive(Debug)]
pub struct PatchContext {
    pub left: Value,
    pub delta: Delta,
    pub options: Options,
}

impl PatchContext {
    pub fn new(left: Value, delta: Delta) -> Self {
        Self {
            left,
            delta,
            options: Options::default(),
        }
    }
}

#[derive(Debug)]
pub struct ReverseContext {
    pub delta: Delta,
    pub options: Options,
}

impl ReverseContext {
    pub fn new(delta: Delta) -> Self {
        Self {
            delta,
            options: Options::default(),
        }
    }
}