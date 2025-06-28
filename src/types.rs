use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

const MAGIC_NUMBER_DELETED: u32 = 0;
const MAGIC_NUMBER_UNDEFINED_DIFF: u32 = 2;
const MAGIC_NUMBER_ARRAY_MOVED: u32 = 3;

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

#[derive(Debug, Clone)]
pub enum ArrayDeltaIndex {
    NewOrModified(usize),  // index are in-place (previous or new index)
    RemovedOrMoved(usize), // index are the old index
}

#[derive(Debug, Clone)]
pub enum Delta {
    Added(Value),
    Modified(Value, Value),
    Deleted(Value),
    Object(HashMap<String, Delta>),
    Array(Vec<(ArrayDeltaIndex, Delta)>),
    Moved {
        moved_value: Option<Value>,
        new_index: usize,
    },
    TextDiff(String),
    None,
}

impl Delta {
    pub fn to_serializable(self) -> Value {
        match self {
            Delta::Added(new_value) => Value::Array(vec![new_value]),
            Delta::Modified(old_value, new_value) => Value::Array(vec![old_value, new_value]),
            Delta::Deleted(deleted) => {
                Value::Array(vec![deleted, 0.into(), MAGIC_NUMBER_DELETED.into()])
            }
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
                    let key = match index {
                        ArrayDeltaIndex::NewOrModified(index) => {
                            format!("{}", index)
                        }
                        ArrayDeltaIndex::RemovedOrMoved(index) => {
                            format!("_{}", index)
                        }
                    };
                    changes.insert(key, delta.to_serializable());
                }
                Value::Object(changes)
            }
            Delta::Moved {
                moved_value,
                new_index,
            } => Value::Array(vec![
                moved_value.unwrap_or("".into()),
                new_index.into(),
                MAGIC_NUMBER_ARRAY_MOVED.into(),
            ]),
            Delta::TextDiff(uni_diff) => Value::Array(vec![uni_diff.into(), 0.into(), 2.into()]),
            Delta::None => {
                panic!("Delta::None is not serializable");
                // Value::Null
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
    let delta = Delta::Object(HashMap::from([
        ("a".to_string(), (Delta::Added("added".into()))),
        (
            "b".to_string(),
            (Delta::Modified("old".into(), "new".into())),
        ),
        ("c".to_string(), (Delta::Deleted("deleted".into()))),
        (
            "d".to_string(),
            (Delta::Moved {
                moved_value: Some("moved".into()),
                new_index: 1,
            }),
        ),
        ("e".to_string(), (Delta::TextDiff("text_diff".into()))),
        (
            "f".to_string(),
            (Delta::Array(vec![
                (
                    ArrayDeltaIndex::NewOrModified(5),
                    (Delta::Added("added".into())),
                ),
                (
                    ArrayDeltaIndex::RemovedOrMoved(7),
                    (Delta::Deleted("deleted".into())),
                ),
                (
                    ArrayDeltaIndex::RemovedOrMoved(8),
                    (Delta::Moved {
                        moved_value: Some("moved".into()),
                        new_index: 1,
                    }),
                ),
            ])),
        ),
        (
            "g".to_string(),
            (Delta::Object(HashMap::from([
                ("h".to_string(), (Delta::Added("added".into()))),
                (
                    "i".to_string(),
                    (Delta::Modified("old".into(), "new".into())),
                ),
                ("j".to_string(), (Delta::Deleted("deleted".into()))),
                (
                    "k".to_string(),
                    (Delta::Moved {
                        moved_value: Some("moved".into()),
                        new_index: 1,
                    }),
                ),
                ("l".to_string(), (Delta::TextDiff("text_diff".into()))),
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
