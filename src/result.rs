use crate::{query::DiscreteQuery, Shape};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;


#[derive(Default, Serialize, Deserialize)]
pub struct DiscreteShapes {
    pub points:    Vec<[f32; 3]>,
    pub polylines: Vec<Vec<f32>>,
    pub meshes:    Vec<Mesh>, 
}

#[derive(Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vector: Vec<f32>, 
    pub trivec: Vec<usize>,
}

#[wasm_bindgen]
pub fn get_shapes(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let mut result = DiscreteShapes::default();
    for part in query.model.get_shapes() {
        match &part {
            Shape::Point(m) => result.points.push(m.to_array()),
            Shape::Curve(m) => result.polylines.push(m.get_polyline(&query)),
            Shape::Facet(m) => result.meshes.push(m.get_mesh(&query)),
        }
    }
    Ok(serde_wasm_bindgen::to_value(&result)?)
}