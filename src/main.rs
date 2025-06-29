use jsondiffpatch::{
    diff, patch,
    types::{ArrayDeltaIndex, Delta},
};
use serde_json::json;

fn main() {
    env_logger::init();

    println!("JSON Diff Patch Rust - Demo");
    println!("===========================");

    // Example 1: Simple object diff
    let left = if std::env::args().len() > 2 {
        let file = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
        serde_json::from_str(&file).unwrap()
    } else {
        json!({
            "name": "John",
            "age": 30,
            "city": "New York"
        })
    };

    let right = if std::env::args().len() > 2 {
        let file = std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
        serde_json::from_str(&file).unwrap()
    } else {
        json!({
            "name": "John",
            "age": 31,
            "city": "Boston"
        })
    };

    println!("\nExample 1: Object diff");
    println!("Left:  {}", left);
    println!("Right: {}", right);

    if let Some(delta) = diff(&left, &right) {
        println!("Delta: {:#?}", delta);
        // try apply the delta
        let patched = patch(&left, delta).unwrap();
        println!("Patched: {:?}", patched);
        println!(
            "Patched: {}",
            serde_json::to_string_pretty(&patched).unwrap()
        );
        println!("Right: {}", serde_json::to_string_pretty(&right).unwrap());
        assert_eq!(patched, right);
    } else {
        println!("No differences found or diff not implemented yet");
    }

    // Example 2: Array diff
    let left_array = json!([1, 2, 3, 4, 5, 6, 7]);
    let right_array = json!([1, 2, 4, 5, 3, 6, 7]);

    println!("\nExample 2: Array diff");
    println!("Left:  {}", left_array);
    println!("Right: {}", right_array);

    if let Some(delta) = diff(&left_array, &right_array) {
        println!("Delta: {:?}", delta);
        let patched = patch(&left_array, delta).unwrap();
        println!("Patched: {:?}", patched);
        assert_eq!(patched, right_array);
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
    let cloned = original.clone();
    println!("Original: {}", original);
    println!("Cloned:   {}", cloned);
    println!("Equal:    {}", original == cloned);

    // test patch
    let left = json!([1, 2, 3]);
    let a = json!("a");
    let b = json!("b");
    // let c = json!("c");
    let delta = Delta::Array(vec![
        (ArrayDeltaIndex::NewOrModified(0), Delta::Added(&a)),
        (ArrayDeltaIndex::RemovedOrMoved(1), Delta::Deleted(&b)),
        (
            ArrayDeltaIndex::RemovedOrMoved(0),
            Delta::Moved {
                new_index: 2,
                moved_value: None,
            },
        ),
    ]);

    let patch = patch(&left, delta);
    println!("Left: {:?}", left);
    println!("Patch: {:?}", patch);
}
