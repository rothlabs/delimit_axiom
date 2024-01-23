//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

// #[derive(Default, Serialize, Deserialize)]
// #[serde(default="TestNurbsResult::default")]
// struct TestNurbsResult {
//     nurbs: Nurbs,
//     vectors: Vec<Vec<f32>>,
//     basis: Vec<Vec<f32>>,
// }

// #[wasm_bindgen]
// pub fn test_nurbs(val: JsValue) -> Result<JsValue, JsValue> {
//     let queried: VectorQuery = serde_wasm_bindgen::from_value(val)?; // <[f32;3]>
//     let result = TestNurbsResult {
//         nurbs: Nurbs {
//             order:   queried.nurbs.get_order(),
//             knots:   queried.nurbs.get_knots(),
//             weights: queried.nurbs.get_weights(),
//             ..Default::default()
//         },
//         vectors: queried.nurbs.get_vectors(queried.count),
//         basis: queried.nurbs.get_basis_plot_vectors(queried.control_index, queried.count),
//     };
//     Ok(serde_wasm_bindgen::to_value(&result)?)
// }
