use crate::context::{DiffContext, FilterContext};
use crate::errors::JsonDiffPatchError;
use crate::types::Delta;
use diff_match_patch_rs::{DiffMatchPatch, Efficient, PatchInput};
use once_cell::sync::Lazy;

const DEFAULT_MIN_LENGTH: usize = 60;

pub struct PatchPipeline;

pub(crate) static DMP: Lazy<DiffMatchPatch> = Lazy::new(DiffMatchPatch::new);

pub fn process_text_diff<'a>(
    context: &mut DiffContext<'a>,
    left: &str,
    right: &str,
) -> Result<(), JsonDiffPatchError> {
    // Get minimum length from options or use default
    let min_length = context
        .options()
        .text_diff
        .as_ref()
        .and_then(|td| td.min_length)
        .unwrap_or(DEFAULT_MIN_LENGTH);

    // If strings are too short, use regular string replace
    if left.len() < min_length || right.len() < min_length {
        context
            .set_result(Delta::Modified(context.left, context.right))
            .exit();
        return Ok(());
    }

    // Try to use text-diff algorithm
    let diffs = DMP.diff_main::<Efficient>(left, right)?;
    // Now, we are going to create a list of `patches` to be applied to the old text to get the new text
    let patches = DMP.patch_make(PatchInput::new_diffs(&diffs))?;
    // in the real world you are going to transmit or store this diff serialized to undiff format to be consumed or used somewhere elese
    let patch_txt = DMP.patch_to_text(&patches);

    context.set_result(Delta::TextDiff(patch_txt)).exit();
    Ok(())
}
