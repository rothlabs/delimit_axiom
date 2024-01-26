
use super::Model;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "PolylineQuery::default")]
struct PolylineQuery {
    model: Model,
    count: usize,
}

#[wasm_bindgen]
pub fn get_polyline(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: PolylineQuery = serde_wasm_bindgen::from_value(val)?; 
    let count = queried.count.clamp(2, 10000);
    let vector = get_polyline_vector(queried.model, count);
    Ok(serde_wasm_bindgen::to_value(&vector)?)
}

fn get_polyline_vector(model: Model, count: usize) -> Vec<f32> {
    match model {
        Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_v(0., count),
        Model::SliceAtU(slice) => slice.get_polyline_at_u(count),
        Model::SliceAtV(slice) => slice.get_polyline_at_v(count),
        _ => vec![0.; 6],
    }
}