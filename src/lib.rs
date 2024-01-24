mod utils;
mod vector;
mod nurbs;
mod mesh;

//use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use nurbs::*;
// use js_sys::Set;
// use gloo_utils::format::JsValueSerdeExt;


#[derive(Clone, Serialize, Deserialize)] //#[serde(default = "Control::default")]
pub enum Model {
    Vector(Vec<f32>),
    Nurbs(Nurbs),
    SliceAtU(Slice),
    SliceAtV(Slice),
    // SliceByPlane(Slice),
    // Intersection(???),
}

impl Default for Model {
    fn default() -> Self { Model::Vector(vec![0.;3]) }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Slice::default")]
pub struct Slice {
    models: Vec<Model>,
    t: f32,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "PolygonQuery::default")]
struct PolygonQuery {
    model: Model,
    count: usize,
}

#[wasm_bindgen]
pub fn get_polygon(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: PolygonQuery = serde_wasm_bindgen::from_value(val)?; // <[f32;3]>
    let count = queried.count.clamp(2, 10000);
    let vectors = get_vectors(queried.model, count);//queried.nurbs.get_valid().get_curve_vectors(count); // get_discrete_vector(100); // of the form [0,0,0,  0,0,0,  0,0,0, ...] // get_sequence_vector
    Ok(serde_wasm_bindgen::to_value(&vectors)?)
}

fn get_vectors(model: Model, count: usize) -> Vec<Vec<f32>> {
    match model {
        Model::Nurbs(nurbs) => nurbs.get_valid().get_vectors_at_v(0., count),
        Model::SliceAtU(slice) => slice.get_vectors_at_u(count), //get_slice_at_u_vectors(&slice.models[0], slice.t, count),//.get_valid().get_vectors_at_u(slice.t, count),
        Model::SliceAtV(slice) => slice.get_vectors_at_v(count), //Model::SliceAtV(slice) => slice.models//.get_valid().get_vectors_at_v(slice.t, count),
        _ => vec![vec![0.; 3]; 2],
    }
}

impl Slice {
    fn get_vectors_at_u(&self, count: usize) -> Vec<Vec<f32>> {
        match &self.models[0] {
            Model::Nurbs(nurbs) => nurbs.get_valid().get_vectors_at_u(self.t, count),
            _ => vec![vec![0.; 3]; 2],
        }
    }
    fn get_vectors_at_v(&self, count: usize) -> Vec<Vec<f32>> {
        match &self.models[0] {
            Model::Nurbs(nurbs) => nurbs.get_valid().get_vectors_at_v(self.t, count),
            _ => vec![vec![0.; 3]; 2],
        }
    }
}

// fn get_slice_at_u_vectors(model: &Model, count: usize) -> Vec<Vec<f32>>{
//     match model {
//         Model::Nurbs(nurbs) => nurbs.get_valid().get_vectors_at_u(0., count),
//         _ => vec![vec![0.; 3]; 2],
//     }
// }



// #[wasm_bindgen]
// pub fn test_enum(val: JsValue) -> Result<JsValue, JsValue> {
//     let queried: Control = serde_wasm_bindgen::from_value(val)?; 
//     //let result = queried.nurbs.get_vectors(count); 
//     Ok(serde_wasm_bindgen::to_value(&queried)?)
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
