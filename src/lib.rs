mod utils;
mod vector;
mod nurbs;

//use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use nurbs::*;
//use vector::*;
// use js_sys::Set;
// use gloo_utils::format::JsValueSerdeExt;

#[derive(Default, Serialize, Deserialize)]
#[serde(default="VectorQuery::default")]
struct VectorQuery {
    count: usize,
    nurbs: Nurbs, 
    control_index: usize,
}

#[wasm_bindgen]
pub fn get_vectors(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: VectorQuery = serde_wasm_bindgen::from_value(val)?; // <[f32;3]>
    let count = queried.count.clamp(2, 10000);
    let result = queried.nurbs.get_vectors(count); // get_discrete_vector(100); // of the form [0,0,0,  0,0,0,  0,0,0, ...] // get_sequence_vector
    Ok(serde_wasm_bindgen::to_value(&result)?)
}


#[derive(Default, Serialize, Deserialize)]
#[serde(default="TestNurbsResult::default")]
struct TestNurbsResult {
    nurbs: Nurbs,
    vectors: Vec<Vec<f32>>,
    basis: Vec<Vec<f32>>,
}

#[wasm_bindgen]
pub fn test_nurbs(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: VectorQuery = serde_wasm_bindgen::from_value(val)?; // <[f32;3]>
    let result = TestNurbsResult {
        nurbs: Nurbs {
            order:   queried.nurbs.get_order(),
            knots:   queried.nurbs.get_knots(),
            weights: queried.nurbs.get_weights(),
            ..Default::default()
        },
        vectors: queried.nurbs.get_vectors(queried.count),
        basis: queried.nurbs.get_basis_plot_vectors(queried.control_index, queried.count),
    };
    Ok(serde_wasm_bindgen::to_value(&result)?)
}


    //let new_burbs = Nurbs::default();
    //let result = nurbs.get_knots();//nurbs_query.get_vectors()?; //[vector.x, vector.y, vector.z];


// example.field1.insert(String::from("awesome"), String::from("more stuff"));


    // let vector1 = Vector3::default();
    // let vector2 = Vector3::default();
    // let nurbs = Nurbs {
    //     order: 2,
    //     knots: vec![0.0, 0.0, 0.0, 0.0],
    //     weights: vec![0.0, 0.0],
    //     vectors: vec![vector1, vector2],
    // };



// pub struct Example {
//     pub field1: HashMap<String, String>,
//     pub field2: Vec<Vec<f32>>,
//     pub field3: [f32; 4],
// }
// #[wasm_bindgen]
// pub fn get_vectors(val: JsValue) -> Result<JsValue, JsValue> {
//     let mut example: Example = serde_wasm_bindgen::from_value(val)?;
//     example.field1.insert(String::from("awesome"), String::from("more stuff"));
//     // let mut field1 = HashMap::new();
//     // field1.insert(0, String::from("ex"));
//     // let example = Example {
//     //     field1,
//     //     field2: vec![vec![1., 2.], vec![3., 4.]],
//     //     field3: [1., 2., 3., 4.]
//     // };
//     Ok(serde_wasm_bindgen::to_value(&example)?)
// }




// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet() {
//     alert("Hello, delimit_axiom!");
// }

// #[wasm_bindgen]
// pub fn add(a:f32, b:f32) -> f32 {
//     a + b
// }




// #[wasm_bindgen]
// pub fn add_wasm_by_example_to_string() -> String {
// "hello".into()//let result = format!("{} {}", input_string, "Wasm by Example");
//   //return result.into();
// }



// // #[wasm_bindgen]
// // pub fn take_number_slice_by_shared_ref(x: &[f32]) -> f32 {
// //     56.777
// // }


// #[wasm_bindgen]
// pub fn return_boxed_js_value_slice() -> Box<[JsValue]> {
//     vec![JsValue::NULL, JsValue::UNDEFINED].into_boxed_slice()
// }

// // #[wasm_bindgen]
// // pub fn count_strings_in_set(set: &js_sys::Set) -> u32 {
// //     let mut count = 0;

// //     // Call `keys` to get an iterator over the set's elements. Because this is
// //     // in a `for ... in ...` loop, Rust will automatically call its
// //     // `IntoIterator` trait implementation to convert it into a Rust iterator.
// //     for x in set.keys() {
// //         // We know the built-in iterator for set elements won't throw
// //         // exceptions, so just unwrap the element. If this was an untrusted
// //         // iterator, we might want to explicitly handle the case where it throws
// //         // an exception instead of returning a `{ value, done }` object.
// //         let x = x.unwrap();

// //         // If `x` is a string, increment our count of strings in the set!
// //         if x.is_string() {
// //             count += 1;
// //         }
// //     }

// //     count
// // }