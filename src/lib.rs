pub mod types;
pub mod context;
pub mod processor;
pub mod diffpatcher;
pub mod filters;
pub mod clone;
pub mod date_reviver;

pub use diffpatcher::DiffPatcher;
pub use date_reviver::date_reviver;
pub use types::{Delta, Options};

use std::sync::OnceLock;

static DEFAULT_INSTANCE: OnceLock<DiffPatcher> = OnceLock::new();

pub fn create(options: Option<Options>) -> DiffPatcher {
    DiffPatcher::new(options)
}

pub fn diff(left: &serde_json::Value, right: &serde_json::Value) -> Option<Delta> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.diff(left, right)
}

pub fn patch(left: &serde_json::Value, delta: &Delta) -> Option<serde_json::Value> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.patch(left, delta)
}

pub fn unpatch(right: &serde_json::Value, delta: &Delta) -> Option<serde_json::Value> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.unpatch(right, delta)
}

pub fn reverse(delta: &Delta) -> Option<Delta> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.reverse(delta)
}

pub fn clone(value: &serde_json::Value) -> serde_json::Value {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.clone(value)
}