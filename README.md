# jsondiffpatch-rs

A Rust implementation of JSON diff & patch functionality, providing object and array diff, text diff, and multiple output formats.

This is a Rust clone of the original [jsondiffpatch](https://github.com/benjamine/jsondiffpatch) TypeScript library.

## Features

- **Object Diffing**: Compare JSON objects and generate deltas
- **Array Diffing**: Detect array changes including moves, inserts, and deletes
- **Text Diffing**: Character-level text comparison using diff-match-patch
- **Pipeline Architecture**: Modular filter-based processing pipeline
- **Delta Operations**: Create, apply, and reverse deltas
- **Error Handling**: Comprehensive error types for robust operation

## Quick Start

```rust
use jsondiffpatch_rs::DiffPatcher;
use serde_json::json;

fn main() {
    // Create a diff patcher instance
    let diffpatcher = DiffPatcher::new(None);

    // Create two JSON objects to compare
    let left = json!({
        "name": "John",
        "age": 30,
        "city": "New York"
    });

    let right = json!({
        "name": "John",
        "age": 31,
        "city": "Boston"
    });

    // Generate a delta
    if let Some(delta) = diffpatcher.diff(&left, &right) {
        println!("Delta: {:?}", delta);

        // Apply the delta to get the right object
        if let Some(patched) = diffpatcher.patch(&left, delta) {
            println!("Patched: {}", patched);
        }
    }
}
```

## API Reference

### Core Types

#### `DiffPatcher`
Main entry point for diff and patch operations.

```rust
let diffpatcher = DiffPatcher::new(Some(options));
```

#### `Delta<'a>`
Represents changes between JSON values:

- `Delta::Added(&'a Value)` - A new value was added
- `Delta::Modified(&'a Value, &'a Value)` - A value was changed
- `Delta::Deleted(&'a Value)` - A value was removed
- `Delta::Object(HashMap<String, Delta<'a>>)` - Object property changes
- `Delta::Array(Vec<(ArrayDeltaIndex, Delta<'a>)>)` - Array element changes
- `Delta::Moved { moved_value: Option<&'a Value>, new_index: usize }` - Array element was moved
- `Delta::TextDiff(String)` - Text-level changes
- `Delta::None` - No changes

#### `Options`
Configuration options for the diffing process:

```rust
use jsondiffpatch_rs::types::{Options, ArrayOptions, TextDiffOptions};

let options = Options {
    match_by_position: Some(false),
    arrays: Some(ArrayOptions {
        detect_move: Some(true),
        include_value_on_move: Some(false),
    }),
    text_diff: Some(TextDiffOptions {
        min_length: Some(60),
    }),
    clone_diff_values: Some(false),
    omit_removed_values: Some(false),
};
```

### Main Methods

#### `diff(left: &'a Value, right: &'a Value) -> Option<Delta<'a>>`
Generates a delta representing the differences between two JSON values.

#### `patch(left: &Value, delta: Delta) -> Option<Value>`
Applies a delta to a JSON value to produce the target value.

#### `reverse(delta: &Delta) -> Option<Delta>`
Reverses a delta to create an inverse delta.

#### `unpatch(right: &Value, delta: &Delta) -> Option<Value>`
Reverses a delta to get the original value from the target value.

## Examples

### Object Diffing

```rust
use jsondiffpatch_rs::DiffPatcher;
use serde_json::json;

let diffpatcher = DiffPatcher::new(None);
let left = json!({"a": 1, "b": 2});
let right = json!({"a": 1, "b": 3, "c": 4});

if let Some(delta) = diffpatcher.diff(&left, &right) {
    // delta will contain changes for "b" and "c"
    println!("Delta: {:?}", delta);
}
```

### Array Diffing

```rust
let left = json!([1, 2, 3]);
let right = json!([1, 4, 2, 3]);

if let Some(delta) = diffpatcher.diff(&left, &right) {
    // delta will contain array changes including moves
    println!("Array delta: {:?}", delta);
}
```

### Text Diffing

```rust
let left = json!("hello world");
let right = json!("hello rust world");

if let Some(delta) = diffpatcher.diff(&left, &right) {
    // delta will contain text-level changes using diff-match-patch
    println!("Text delta: {:?}", delta);
}
```

### Applying Patches

```rust
let left = json!({"name": "John", "age": 30});
let delta = diffpatcher.diff(&left, &right).unwrap();

if let Some(patched) = diffpatcher.patch(&left, delta) {
    assert_eq!(patched, right);
}
```

## Architecture

The library follows a pipeline-based architecture:

### Core Components

- **Processor**: Manages the processing pipeline and context flow
- **Pipeline**: Defines the interface for processing operations (diff, patch, reverse)
- **Contexts**: Carry data through the processing pipeline
  - `DiffContext`: Handles diff operations
  - `PatchContext`: Handles patch operations
  - `ReverseContext`: Handles reverse operations
- **Filters**: Handle specific types of data processing

### Pipeline Structure

```rust
// Diff pipeline
DiffPipeline -> [ObjectFilter, ArrayFilter, TextFilter, ...]

// Patch pipeline
PatchPipeline -> [ObjectFilter, ArrayFilter, TextFilter, ...]
```

### Error Handling

The library provides comprehensive error handling through `JsonDiffPatchError`:

```rust
use jsondiffpatch_rs::errors::JsonDiffPatchError;

match diffpatcher.diff(&left, &right) {
    Some(delta) => println!("Success: {:?}", delta),
    None => println!("No differences found"),
}
```

## Delta Format

The library uses a compact delta format compatible with the original jsondiffpatch:

### Serialization

Deltas can be converted to JSON format for storage/transmission:

```rust
if let Some(delta) = diffpatcher.diff(&left, &right) {
    let serialized = delta.to_serializable();
    println!("Serialized delta: {}", serialized);
}
```

### Magic Numbers

The library uses magic numbers to identify special operations:
- `0`: Deleted items
- `2`: Text diff operations
- `3`: Array move operations

## Status

Current implementation status:

- ✅ Core architecture and pipeline system
- ✅ Basic types and error handling
- ✅ Object diffing and patching
- ✅ Array diffing with move detection
- ✅ Text diffing using diff-match-patch
- ✅ Delta serialization and deserialization
- ✅ Comprehensive test coverage
- 🔄 Reverse operations (planned)
- 🔄 Performance optimizations (ongoing)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see the LICENSE file for details.

## Acknowledgments

This project is based on the original [jsondiffpatch](https://github.com/benjamine/jsondiffpatch) library
