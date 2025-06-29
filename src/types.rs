use crate::errors::JsonDiffPatchReverseError;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

const MIDDLE_NO_VALUE: u32 = 0;

pub enum MagicNumber {
    Deleted = 0,
    UndefinedDiff = 2,
    ArrayMoved = 3,
}

impl From<MagicNumber> for Value {
    fn from(magic: MagicNumber) -> Value {
        Value::Number(serde_json::Number::from(magic as u32))
    }
}

#[derive(Clone)]
pub struct Options {
    // pub object_hash: Option<Box<dyn Fn(&Value, Option<usize>) -> Option<String> + Send + Sync>>,
    pub match_by_position: Option<bool>,
    pub arrays: Option<ArrayOptions>,
    pub text_diff: Option<TextDiffOptions>,
    // pub property_filter: Option<Box<dyn Fn(&str, &DiffContext) -> bool + Send + Sync + 'static>>,
    pub clone_diff_values: Option<bool>,
    pub omit_removed_values: Option<bool>,
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            // .field(
            //     "object_hash",
            //     &if self.object_hash.is_some() {
            //         "Some(Fn)"
            //     } else {
            //         "None"
            //     },
            // )
            .field("match_by_position", &self.match_by_position)
            .field("arrays", &self.arrays)
            .field("text_diff", &self.text_diff)
            // .field(
            //     "property_filter",
            //     &if self.property_filter.is_some() {
            //         "Some(Fn)"
            //     } else {
            //         "None"
            //     },
            // )
            .field("clone_diff_values", &self.clone_diff_values)
            .field("omit_removed_values", &self.omit_removed_values)
            .finish()
    }
}

pub(crate) static OPTIONS: OnceCell<Options> = OnceCell::new();

#[derive(Debug, Clone)]
pub enum ArrayDeltaIndex {
    NewOrModified(usize),  // index are in-place (previous or new index)
    RemovedOrMoved(usize), // index are the old index
}

impl ArrayDeltaIndex {
    pub fn to_serializable(&self) -> String {
        match self {
            ArrayDeltaIndex::NewOrModified(index) => {
                format!("{}", index)
            }
            ArrayDeltaIndex::RemovedOrMoved(index) => {
                format!("_{}", index)
            }
        }
    }
}

impl PartialEq for ArrayDeltaIndex {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal) == std::cmp::Ordering::Equal
    }
}

impl Eq for ArrayDeltaIndex {}

impl PartialOrd for ArrayDeltaIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArrayDeltaIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ArrayDeltaIndex::NewOrModified(a), ArrayDeltaIndex::NewOrModified(b)) => a.cmp(b),
            (ArrayDeltaIndex::RemovedOrMoved(a), ArrayDeltaIndex::RemovedOrMoved(b)) => a.cmp(b),
            (ArrayDeltaIndex::RemovedOrMoved(_), ArrayDeltaIndex::NewOrModified(_)) => {
                std::cmp::Ordering::Less
            }
            (ArrayDeltaIndex::NewOrModified(_), ArrayDeltaIndex::RemovedOrMoved(_)) => {
                std::cmp::Ordering::Greater
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Delta<'a> {
    Added(&'a Value),
    Modified(&'a Value, &'a Value),
    Deleted(&'a Value),
    Object(HashMap<String, Delta<'a>>),
    Array(Vec<(ArrayDeltaIndex, Delta<'a>)>),
    Moved {
        moved_value: Option<&'a Value>,
        new_index: usize,
    },
    TextDiff(String),
    None,
}

impl<'a> Delta<'a> {
    pub fn to_serializable(self) -> Value {
        match self {
            Delta::Added(new_value) => Value::Array(vec![new_value.clone()]),
            Delta::Modified(old_value, new_value) => {
                Value::Array(vec![old_value.clone(), new_value.clone()])
            }
            Delta::Deleted(deleted) => Value::Array(vec![
                deleted.clone(),
                MIDDLE_NO_VALUE.into(),
                MagicNumber::Deleted.into(),
            ]),
            Delta::Object(value) => Value::Object(
                value
                    .into_iter()
                    .map(|(k, v)| (k, v.to_serializable()))
                    .collect(),
            ),
            Delta::Array(array_changes) => {
                let mut changes = ::serde_json::Map::new();
                // marker
                changes.insert("_t".to_string(), Value::String("a".to_string()));
                for (index, delta) in array_changes {
                    changes.insert(index.to_serializable(), delta.to_serializable());
                }
                Value::Object(changes)
            }
            Delta::Moved {
                moved_value,
                new_index,
            } => Value::Array(vec![
                moved_value.unwrap_or(&Value::Null).clone(),
                new_index.into(),
                MagicNumber::ArrayMoved.into(),
            ]),
            Delta::TextDiff(uni_diff) => Value::Array(vec![
                uni_diff.into(),
                MIDDLE_NO_VALUE.into(),
                MagicNumber::UndefinedDiff.into(),
            ]),
            Delta::None => {
                panic!("Delta::None is not serializable");
                // Value::Null
            }
        }
    }

    /// Reverses the delta
    pub fn build_reverse(self) -> Result<Delta<'a>, JsonDiffPatchReverseError> {
        match self {
            Delta::Added(new_value) => Ok(Delta::Deleted(new_value)),
            Delta::Modified(old_value, new_value) => Ok(Delta::Modified(new_value, old_value)),
            Delta::Deleted(deleted) => Ok(Delta::Added(deleted)),
            Delta::Object(object) => {
                let mut reversed_changes = HashMap::new();
                for (key, value) in object {
                    reversed_changes.insert(key, value.build_reverse()?);
                }
                Ok(Delta::Object(reversed_changes))
            }
            Delta::None => Ok(Delta::None),
            Delta::Moved {
                moved_value: _,
                new_index: _,
            } => Err(JsonDiffPatchReverseError::InvalidMoveDelta),
            Delta::TextDiff(uni_diff) => {
                todo!()
            }
            Delta::Array(array) => {
                todo!()
            }
        }
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
            // object_hash: None,
            match_by_position: Some(false),
            arrays: Some(ArrayOptions {
                detect_move: Some(true),
                include_value_on_move: Some(false),
            }),
            text_diff: Some(TextDiffOptions {
                min_length: Some(60),
            }),
            // property_filter: None,
            clone_diff_values: Some(false),
            omit_removed_values: Some(false),
        }
    }
}

#[test]
fn test_my_delta_to_serializable() {
    let added = "added".into();
    let old = "old".into();
    let new = "new".into();
    let deleted = "deleted".into();
    let moved = "moved".into();
    let text_diff = "text_diff";

    let delta = Delta::Object(HashMap::from([
        ("a".to_string(), (Delta::Added(&added))),
        ("b".to_string(), (Delta::Modified(&old, &new))),
        ("c".to_string(), (Delta::Deleted(&deleted))),
        (
            "d".to_string(),
            (Delta::Moved {
                moved_value: Some(&moved),
                new_index: 1,
            }),
        ),
        ("e".to_string(), (Delta::TextDiff(text_diff.to_string()))),
        (
            "f".to_string(),
            (Delta::Array(vec![
                (ArrayDeltaIndex::NewOrModified(5), (Delta::Added(&added))),
                (
                    ArrayDeltaIndex::RemovedOrMoved(7),
                    (Delta::Deleted(&deleted)),
                ),
                (
                    ArrayDeltaIndex::RemovedOrMoved(8),
                    (Delta::Moved {
                        moved_value: Some(&moved),
                        new_index: 1,
                    }),
                ),
            ])),
        ),
        (
            "g".to_string(),
            (Delta::Object(HashMap::from([
                ("h".to_string(), (Delta::Added(&added))),
                ("i".to_string(), (Delta::Modified(&old, &new))),
                ("j".to_string(), (Delta::Deleted(&deleted))),
                (
                    "k".to_string(),
                    (Delta::Moved {
                        moved_value: Some(&moved),
                        new_index: 1,
                    }),
                ),
                ("l".to_string(), (Delta::TextDiff(text_diff.to_string()))),
            ]))),
        ),
    ]));

    assert_eq!(
        delta.to_serializable().to_string(),
        r#"{
            "a":["added"],
            "b":["old","new"],
            "c":["deleted",0,0],
            "d":["moved",1,3],
            "e":["text_diff",0,2],
            "f":{
                "5":["added"],
                "_7":["deleted",0,0],
                "_8":["moved",1,3],
                "_t":"a"
            },
            "g":{
                "h":["added"],
                "i":["old","new"],
                "j":["deleted",0,0],
                "k":["moved",1,3],
                "l":["text_diff",0,2]
            }
        }"#
        .replace("\n", "")
        .replace(" ", ""),
    );
}
