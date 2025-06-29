pub mod context;
pub mod diffpatcher;
pub mod errors;
pub mod filters;
pub mod lcs;
pub mod processor;
pub mod types;

pub use diffpatcher::DiffPatcher;
pub use types::{Delta, Options};

use std::sync::OnceLock;

static DEFAULT_INSTANCE: OnceLock<DiffPatcher> = OnceLock::new();

pub fn create(options: Option<Options>) -> DiffPatcher {
    DiffPatcher::new(options)
}

pub fn diff<'a>(left: &'a serde_json::Value, right: &'a serde_json::Value) -> Option<Delta<'a>> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.diff(left, right)
}

pub fn patch(left: &serde_json::Value, delta: Delta) -> Option<serde_json::Value> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.patch(left, delta)
}

pub fn unpatch(right: &serde_json::Value, delta: &Delta) -> Option<serde_json::Value> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.unpatch(right, delta)
}

pub fn reverse<'a>(delta: &Delta<'a>) -> Option<Delta<'a>> {
    let instance = DEFAULT_INSTANCE.get_or_init(|| DiffPatcher::new(None));
    instance.reverse(delta)
}
