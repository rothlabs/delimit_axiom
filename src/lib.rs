mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, delimit_axiom!");
}

#[wasm_bindgen]
pub fn add(a:f32, b:f32) -> f32 {
    a + b
}
