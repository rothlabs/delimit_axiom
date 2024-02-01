mod utils;

mod group;
mod nurbs;
mod slice;
mod polyline;
mod mesh;
mod turtled;
mod path;
mod area;
mod extrusion;

use utils::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use group::*;
use nurbs::*;
use slice::*;
use turtled::*;
use path::*;
use area::*;
use extrusion::*;

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
    Vector(Vec<f32>),
    Group(Group),
    Nurbs(Nurbs),
    Slice(Slice),
    Turtled(Turtled),
    Circle(Circle),
    Rectangle(Rectangle),
    Area(Area),
    Extrusion(Extrusion),
    Path(Path),
    MoveTo(Box<Model>),
    LineTo(Box<Model>),
    ArcTo(ArcTo),
    Close(bool), // TODO: find way to remove bool
}

impl Default for Model {
    fn default() -> Self { Model::Vector(vec![0.; 3]) }
}

#[derive(Clone, Serialize, Deserialize)] 
pub enum Parameter {
    U(f32),
    V(f32),
}

impl Default for Parameter {
    fn default() -> Self { Parameter::U(0.) }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "DiscreteQuery::default")]
pub struct DiscreteQuery {
    pub model:     Model,
    pub tolerance: f32,   // allowed distance from real model
    pub count:     usize, // quantity of points from the model (when tolerance is not implemented)
    pub u_count:   usize, // for surfaces
    pub v_count:   usize, // for surfaces
}

#[wasm_bindgen]
pub fn enable_panic_messages() {
    set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}




//use std::collections::HashMap;
// use js_sys::Set;
// use gloo_utils::format::JsValueSerdeExt;



// #[derive(Default, Serialize, Deserialize)]
// #[serde(default = "PolylineQuery::default")]
// struct PolylineQuery {
//     model: Model,
//     count: usize,
// }

// #[wasm_bindgen]
// pub fn get_polyline(val: JsValue) -> Result<JsValue, JsValue> {
//     let queried: PolylineQuery = serde_wasm_bindgen::from_value(val)?; // <[f32;3]>
//     let count = queried.count.clamp(2, 10000);
//     let vector = get_polyline_vector(queried.model, count);//queried.nurbs.get_valid().get_curve_vectors(count); // get_discrete_vector(100); // of the form [0,0,0,  0,0,0,  0,0,0, ...] // get_sequence_vector
//     Ok(serde_wasm_bindgen::to_value(&vector)?)
// }

// fn get_polyline_vector(model: Model, count: usize) -> Vec<f32> {
//     match model {
//         Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_v(0., count),
//         Model::SliceAtU(slice) => slice.get_polyline_at_u(count), //get_slice_at_u_vectors(&slice.models[0], slice.t, count),//.get_valid().get_vectors_at_u(slice.t, count),
//         Model::SliceAtV(slice) => slice.get_polyline_at_v(count), //Model::SliceAtV(slice) => slice.models//.get_valid().get_vectors_at_v(slice.t, count),
//         _ => vec![0.; 6],
//     }
// }

// impl Slice {
//     fn get_polyline_at_u(&self, count: usize) -> Vec<f32> {
//         match &self.models[0] {
//             Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_u(self.t, count),
//             _ => vec![0.; 6],
//         }
//     }
//     fn get_polyline_at_v(&self, count: usize) -> Vec<f32> {
//         match &self.models[0] {
//             Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_v(self.t, count),
//             _ => vec![0.; 6],
//         }
//     }
// }



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
// pub fn greet() {
//     alert("Hello, delimit_axiom!");
// }

// #[wasm_bindgen]
// pub fn add(a:f32, b:f32) -> f32 {
//     a + b
// }
