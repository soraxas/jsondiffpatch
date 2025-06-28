use crate::context::{DiffContext, FilterContext};
use crate::processor::Filter;
use crate::types::Delta;
use diff_match_patch_rs::{DiffMatchPatch, Efficient, PatchInput};

const DEFAULT_MIN_LENGTH: usize = 60;

// static CACHED_DMP: OnceLock<Dmp> = OnceLock::new();

// fn get_diff_match_patch(options: &Options) -> Option<&'static Dmp> {
//     // For now, we'll always use the default DMP instance
//     // In the future, this could be configurable through options
//     CACHED_DMP.get_or_init(|| Dmp::new())
// }

// fn text_delta_reverse(delta: &str) -> String {
//     let header_regex = regex::Regex::new(r"^@@ +-(\d+),(\d+) +\+(\d+),(\d+) +@@$").unwrap();
//     let mut lines: Vec<&str> = delta.split('\n').collect();

//     for i in 0..lines.len() {
//         let line = lines[i];
//         if line.is_empty() {
//             continue;
//         }

//         let line_start = line.chars().next().unwrap_or(' ');

//         if line_start == '@' {
//             if let Some(captures) = header_regex.captures(line) {
//                 // Fix header by swapping the numbers
//                 let new_header = format!(
//                     "@@ -{},{} +{},{} @@",
//                     captures.get(3).unwrap().as_str(),
//                     captures.get(4).unwrap().as_str(),
//                     captures.get(1).unwrap().as_str(),
//                     captures.get(2).unwrap().as_str()
//                 );
//                 lines[i] = &new_header;
//             }
//         } else if line_start == '+' {
//             // Convert + to -
//             lines[i] = &format!("-{}", &line[1..]);

//             // Swap lines to keep default order (-+)
//             if i > 0 && lines[i - 1].starts_with('+') {
//                 lines.swap(i, i - 1);
//             }
//         } else if line_start == '-' {
//             // Convert - to +
//             lines[i] = &format!("+{}", &line[1..]);
//         }
//     }

//     lines.join("\n")
// }

pub struct TextsDiffFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for TextsDiffFilter {
    fn filter_name(&self) -> &str {
        "texts-diff"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        _new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // Check if both values are strings
        if !context.left.is_string() || !context.right.is_string() {
            return;
        }

        let left = context.left.as_str().unwrap();
        let right = context.right.as_str().unwrap();

        // Get minimum length from options or use default
        let min_length = context
            .options
            .text_diff
            .as_ref()
            .and_then(|td| td.min_length)
            .unwrap_or(DEFAULT_MIN_LENGTH);

        // If strings are too short, use regular string replace
        if left.len() < min_length || right.len() < min_length {
            context
                .set_result(Delta::Modified(context.left, context.right))
                .exit();
            return;
        }

        // Try to use text-diff algorithm
        let dmp = DiffMatchPatch::new();

        let diffs = dmp.diff_main::<Efficient>(left, right).unwrap();
        // Now, we are going to create a list of `patches` to be applied to the old text to get the new text
        let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
        // in the real world you are going to transmit or store this diff serialized to undiff format to be consumed or used somewhere elese
        let patch_txt = dmp.patch_to_text(&patches);

        context.set_result(Delta::TextDiff(patch_txt)).exit();
    }

    fn post_process(
        &self,
        _context: &mut DiffContext<'a>,
        _children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // No post-processing needed for text diff
    }
}

pub struct TextsPatchFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for TextsPatchFilter {
    fn filter_name(&self) -> &str {
        "texts-patch"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        _new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // This filter is for patching, not diffing
        // The actual patch logic would be in a separate PatchContext
    }

    fn post_process(
        &self,
        _context: &mut DiffContext<'a>,
        _children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // No post-processing needed
    }
}

pub struct TextsReverseFilter;

impl<'a> Filter<DiffContext<'a>, Delta<'a>> for TextsReverseFilter {
    fn filter_name(&self) -> &str {
        "texts-reverse"
    }

    fn process(
        &self,
        context: &mut DiffContext<'a>,
        _new_children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // This filter is for reversing, not diffing
        // The actual reverse logic would be in a separate ReverseContext
    }

    fn post_process(
        &self,
        _context: &mut DiffContext<'a>,
        _children_context: &mut Vec<(String, DiffContext<'a>)>,
    ) {
        // No post-processing needed
    }
}
