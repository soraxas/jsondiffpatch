use crate::errors::JsonDiffPatchReverseError;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

// const MIDDLE_NO_VALUE: u32 = 0;
const MIDDLE_NO_VALUE: Value = Value::Null;

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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

impl Serialize for ArrayDeltaIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ArrayDeltaIndex::NewOrModified(index) => {
                serializer.serialize_str(&format!("{}", index))
            }
            ArrayDeltaIndex::RemovedOrMoved(index) => {
                serializer.serialize_str(&format!("_{}", index))
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
    Added(Cow<'a, Value>),
    Modified(Cow<'a, Value>, Cow<'a, Value>),
    Deleted(Cow<'a, Value>),
    Object(HashMap<String, Delta<'a>>),
    Array(Vec<(ArrayDeltaIndex, Delta<'a>)>),
    Moved {
        moved_value: Option<Cow<'a, Value>>,
        new_index: usize,
    },
    TextDiff(String),
    None,
}

impl<'a> Delta<'a> {
    pub fn added_ref(value: &'a Value) -> Delta<'a> {
        Delta::Added(Cow::Borrowed(value))
    }

    pub fn modified_ref(old_value: &'a Value, new_value: &'a Value) -> Delta<'a> {
        Delta::Modified(Cow::Borrowed(old_value), Cow::Borrowed(new_value))
    }

    pub fn deleted_ref(value: &'a Value) -> Delta<'a> {
        Delta::Deleted(Cow::Borrowed(value))
    }

    pub fn moved_ref(moved_value: &'a Value, new_index: usize) -> Delta<'a> {
        Delta::Moved {
            moved_value: Some(Cow::Borrowed(moved_value)),
            new_index,
        }
    }

    pub fn text_diff_ref(text_diff: &'a str) -> Delta<'a> {
        Delta::TextDiff(text_diff.to_string())
    }

    // pub fn array(array: Vec<(ArrayDeltaIndex, Delta<'a>)>) -> Delta<'a> {
    //     Delta::Array(array)
    // }

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
            Delta::TextDiff(_uni_diff) => {
                todo!()
            }
            Delta::Array(_array) => {
                todo!()
            }
        }
    }
}

impl Serialize for Delta<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::{SerializeMap, SerializeSeq};

        match self {
            Delta::Added(new_value) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(new_value)?;
                seq.end()
            }
            Delta::Modified(old_value, new_value) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(old_value)?;
                seq.serialize_element(new_value)?;
                seq.end()
            }
            Delta::Deleted(deleted) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(deleted)?;
                seq.serialize_element(&MIDDLE_NO_VALUE)?;
                seq.serialize_element(&MagicNumber::Deleted)?;
                seq.end()
            }
            Delta::Object(value) => {
                let mut map = serializer.serialize_map(Some(value.len()))?;
                for (k, v) in value {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Delta::Array(array_changes) => {
                let mut map = serializer.serialize_map(Some(array_changes.len() + 1))?;
                map.serialize_entry("_t", &Value::String("a".to_string()))?;
                for (index, delta) in array_changes {
                    map.serialize_entry(index, delta)?;
                }
                map.end()
            }
            Delta::Moved {
                moved_value,
                new_index,
            } => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(
                    moved_value.as_ref().unwrap_or(&Cow::Borrowed(&Value::Null)),
                )?;
                seq.serialize_element(new_index)?;
                seq.serialize_element(&MagicNumber::ArrayMoved)?;
                seq.end()
            }
            Delta::TextDiff(uni_diff) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(uni_diff)?;
                seq.serialize_element(&MIDDLE_NO_VALUE)?;
                seq.serialize_element(&MagicNumber::UndefinedDiff)?;
                seq.end()
            }
            Delta::None => {
                panic!("Delta::None is not serializable");
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
    let added: Value = "added".into();
    let old: Value = "old".into();
    let new: Value = "new".into();
    let deleted: Value = "deleted".into();
    let moved: Value = "moved".into();
    let text_diff = "text_diff";

    let delta = Delta::Object(HashMap::from([
        ("a".to_string(), (Delta::added_ref(&added))),
        ("b".to_string(), (Delta::modified_ref(&old, &new))),
        ("c".to_string(), (Delta::deleted_ref(&deleted))),
        (
            "d".to_string(),
            (Delta::Moved {
                moved_value: Some(Cow::Borrowed(&moved)),
                new_index: 1,
            }),
        ),
        ("e".to_string(), (Delta::TextDiff(text_diff.to_string()))),
        (
            "f".to_string(),
            (Delta::Array(vec![
                (
                    ArrayDeltaIndex::NewOrModified(5),
                    (Delta::added_ref(&added)),
                ),
                (
                    ArrayDeltaIndex::RemovedOrMoved(7),
                    (Delta::deleted_ref(&deleted)),
                ),
                (
                    ArrayDeltaIndex::RemovedOrMoved(8),
                    (Delta::Moved {
                        moved_value: Some(Cow::Borrowed(&moved)),
                        new_index: 1,
                    }),
                ),
            ])),
        ),
        (
            "g".to_string(),
            (Delta::Object(HashMap::from([
                ("h".to_string(), (Delta::added_ref(&added))),
                ("i".to_string(), (Delta::modified_ref(&old, &new))),
                ("j".to_string(), (Delta::deleted_ref(&deleted))),
                (
                    "k".to_string(),
                    (Delta::Moved {
                        moved_value: Some(Cow::Borrowed(&moved)),
                        new_index: 1,
                    }),
                ),
                ("l".to_string(), (Delta::TextDiff(text_diff.to_string()))),
            ]))),
        ),
    ]));

    let delta_serialized = serde_json::to_string(&delta).unwrap();

    assert_eq!(
        serde_json::from_str::<Value>(&delta_serialized).unwrap(),
        serde_json::from_str::<Value>(
            r#"{
            "a":["added"],
            "b":["old","new"],
            "c":["deleted",null,0],
            "d":["moved",1,3],
            "e":["text_diff",null,2],
            "f":{
                "5":["added"],
                "_7":["deleted",null,0],
                "_8":["moved",1,3],
                "_t":"a"
            },
            "g":{
                "h":["added"],
                "i":["old","new"],
                "j":["deleted",null,0],
                "k":["moved",1,3],
                "l":["text_diff",null,2]
            }
        }"#
        )
        .unwrap(),
    );
}
