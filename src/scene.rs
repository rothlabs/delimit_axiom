use crate::{get_vector_hash, query::DiscreteQuery, Curve, Facet};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;


#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub points:    Vec<[f32; 3]>,
    pub polylines: Vec<Polyline>,
    pub meshes:    Vec<Mesh>, 
    pub curves:    Vec<Curve>,
    pub facets:    Vec<Facet>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vector: Vec<f32>, 
    pub trivec: Vec<usize>,
    pub digest: u64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Polyline {
    pub vector: Vec<f32>, 
    pub digest: u64,
}

#[wasm_bindgen]
pub fn get_scene(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let mut scene = Scene::default();
    for shape in query.model.get_shapes() {
        match shape.rank {
            0 => scene.points.push(shape.get_point(&[]).to_array()),
            1 => scene.polylines.push(shape.get_polyline(&query)),
            2 => scene.meshes.push(shape.get_mesh(&query)),
            _ => ()
        }
    }
    Ok(serde_wasm_bindgen::to_value(&scene)?)
}

#[wasm_bindgen]
pub fn get_curve_scene(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let mut polylines = vec![];
    for shape in query.model.get_shapes() {
        match shape.rank {
            1 => polylines.push(shape.get_polyline(&query)),
            _ => ()
        }
    }
    Ok(serde_wasm_bindgen::to_value(&polylines)?)
}

#[wasm_bindgen]
pub fn get_facet_scene(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let mut mesh = Mesh::default();
    let mut offset = 0;
    for shape in query.model.get_shapes() {
        if shape.rank == 2 {
            let facet_mesh = shape.get_mesh(&query);
            mesh.vector.extend(&facet_mesh.vector);
            mesh.trivec.extend(facet_mesh.trivec.iter().map(|v| v + offset));
            offset += facet_mesh.vector.len() / 3;
        }
    }
    mesh.digest = get_vector_hash(&mesh.vector);
    Ok(serde_wasm_bindgen::to_value(&mesh)?)
}