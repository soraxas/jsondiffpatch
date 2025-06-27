use jsondiffpatch_rs::{create, diff, patch, unpatch, reverse, clone};
use serde_json::json;

fn main() {
    println!("JSON Diff Patch Rust - Demo");
    println!("===========================");

    // Example 1: Simple object diff
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

    println!("\nExample 1: Object diff");
    println!("Left:  {}", left);
    println!("Right: {}", right);

    if let Some(delta) = diff(&left, &right) {
        println!("Delta: {:?}", delta);
    } else {
        println!("No differences found or diff not implemented yet");
    }

    // Example 2: Array diff
    let left_array = json!([1, 2, 3, 4]);
    let right_array = json!([1, 2, 4, 5]);

    println!("\nExample 2: Array diff");
    println!("Left:  {}", left_array);
    println!("Right: {}", right_array);

    if let Some(delta) = diff(&left_array, &right_array) {
        println!("Delta: {:?}", delta);
    } else {
        println!("No differences found or diff not implemented yet");
    }

    // Example 3: Clone functionality
    println!("\nExample 3: Clone functionality");
    let original = json!({
        "nested": {
            "array": [1, 2, {"object": true}],
            "string": "hello"
        }
    });
    let cloned = clone(&original);
    println!("Original: {}", original);
    println!("Cloned:   {}", cloned);
    println!("Equal:    {}", original == cloned);
}
