use diff_match_patch_rs::Error as DiffMatchPatchError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonDiffPatchError {
    #[error("internal logic error: {0}")]
    InternalPatchLogicError(String),

    #[error("Cannot apply patch: {0}")]
    InvalidPatch(String),

    #[error("The given patch '{patch}' cannot be applied to the target")]
    InvalidPatchToTarget { patch: String },

    #[error("index out of bounds: the patch is trying to remove an item at index {index}, but the array has only {length} items")]
    IndexOutOfBoundsRemove { index: usize, length: usize },
    #[error("index out of bounds: the patch is trying to insert an item at index {index}, but the array has only {length} items")]
    IndexOutOfBoundsInsert { index: usize, length: usize },
    #[error("index out of bounds: the patch is trying to move an item from index {from} to index {to}, but the array has only {length} items")]
    IndexOutOfBoundsMove {
        from: usize,
        to: usize,
        length: usize,
    },
    #[error("index out of bounds: the patch is trying to modify an item at index {index}, but the array has only {length} items")]
    IndexOutOfBoundsModify { index: usize, length: usize },

    #[error("failed to apply text diff: {0:#?}")]
    ApplyTextDiffFailed(DiffMatchPatchError),
}

impl From<DiffMatchPatchError> for JsonDiffPatchError {
    fn from(err: DiffMatchPatchError) -> Self {
        JsonDiffPatchError::ApplyTextDiffFailed(err)
    }
}
