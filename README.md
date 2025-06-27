# jsondiffpatch-rs

A Rust implementation of JSON diff & patch functionality, providing object and array diff, text diff, and multiple output formats.

This is a Rust clone of the original [jsondiffpatch](https://github.com/benjamine/jsondiffpatch) TypeScript library.

## Features

- **Object Diffing**: Compare JSON objects and generate deltas
- **Array Diffing**: Detect array changes including moves, inserts, and deletes
- **Text Diffing**: Character-level text comparison
- **Date Handling**: Special handling for date/time values
- **Delta Operations**: Create, apply, and reverse deltas
- **Deep Cloning**: Efficient deep cloning of JSON values

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jsondiffpatch_rs = "0.1.0"
```

## Quick Start

```rust
use jsondiffpatch_rs::{diff, patch, reverse, clone};
use serde_json::json;

fn main() {
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
    if let Some(delta) = diff(&left, &right) {
        println!("Delta: {:?}", delta);

        // Apply the delta to get the right object
        if let Some(patched) = patch(&left, &delta) {
            println!("Patched: {}", patched);
        }

        // Reverse the delta
        if let Some(reversed) = reverse(&delta) {
            println!("Reversed: {:?}", reversed);
        }
    }
}
```

## API Reference

### Core Functions

#### `diff(left: &Value, right: &Value) -> Option<Delta>`
Generates a delta representing the differences between two JSON values.

#### `patch(left: &Value, delta: &Delta) -> Option<Value>`
Applies a delta to a JSON value to produce the target value.

#### `unpatch(right: &Value, delta: &Delta) -> Option<Value>`
Reverses a delta to get the original value from the target value.

#### `reverse(delta: &Delta) -> Option<Delta>`
Reverses a delta to create an inverse delta.

#### `clone(value: &Value) -> Value`
Creates a deep clone of a JSON value.

### DiffPatcher Class

For more control over the diffing process, you can create a `DiffPatcher` instance:

```rust
use jsondiffpatch_rs::{DiffPatcher, Options};

let options = Options {
    match_by_position: Some(false),
    arrays: Some(ArrayOptions {
        detect_move: Some(true),
        include_value_on_move: Some(false),
    }),
    ..Default::default()
};

let diffpatcher = DiffPatcher::new(Some(options));
let delta = diffpatcher.diff(&left, &right);
```

## Delta Format

The library uses a compact delta format to represent changes:

- **Added**: `Delta::Added(value)` - A new value was added
- **Modified**: `Delta::Modified(old_value, new_value)` - A value was changed
- **Deleted**: `Delta::Deleted(value, 0, 0)` - A value was removed
- **Object**: `Delta::Object(changes)` - Object property changes
- **Array**: `Delta::Array(changes)` - Array element changes
- **Moved**: `Delta::Moved(value, index, 3)` - Array element was moved
- **TextDiff**: `Delta::TextDiff(text, 0, 2)` - Text-level changes

## Examples

### Object Diffing

```rust
let left = json!({"a": 1, "b": 2});
let right = json!({"a": 1, "b": 3, "c": 4});

if let Some(delta) = diff(&left, &right) {
    // delta will contain changes for "b" and "c"
}
```

### Array Diffing

```rust
let left = json!([1, 2, 3]);
let right = json!([1, 4, 2, 3]);

if let Some(delta) = diff(&left, &right) {
    // delta will contain array changes including moves
}
```

### Text Diffing

```rust
let left = json!("hello world");
let right = json!("hello rust world");

if let Some(delta) = diff(&left, &right) {
    // delta will contain text-level changes
}
```

## Architecture

The library follows a filter-based architecture similar to the original TypeScript implementation:

- **Processor**: Manages the processing pipeline
- **Pipes**: Organize filters for different operations (diff, patch, reverse)
- **Filters**: Handle specific types of data (objects, arrays, text, dates)
- **Contexts**: Carry data through the processing pipeline

## Status

This is a work in progress. Currently implemented:

- ✅ Basic structure and types
- ✅ Clone functionality
- ✅ Date handling utilities
- ✅ Filter framework
- 🔄 Core diff/patch algorithms (in progress)
- 🔄 Array move detection (planned)
- 🔄 Text diff algorithms (planned)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see the LICENSE file for details.

## Acknowledgments

This project is based on the original [jsondiffpatch](https://github.com/benjamine/jsondiffpatch) library by Benjamin Eidelman.