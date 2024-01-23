use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
// use super::nurbs::*;
use super::Model;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "MeshQuery::default")]
struct MeshQuery {
    u_count: usize,
    v_count: usize,
    model: Model,
}

#[derive(Serialize, Deserialize)]
struct Mesh {
    vectors: Vec<Vec<f32>>, //Vec<[f32; 3]>,
    indices: Vec<usize>,
}

#[wasm_bindgen]
pub fn get_mesh(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: MeshQuery = serde_wasm_bindgen::from_value(val)?;
    let u_count = queried.u_count.clamp(2, 2000);
    let v_count = queried.v_count.clamp(2, 2000); // queried.nurbs.get_valid().get_surface_vectors(u_count, v_count);
    let result = Mesh {
        vectors: get_vectors(queried.model, u_count, v_count),
        indices: get_indices(u_count, v_count),
    };
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

fn get_vectors(model: Model, u_count: usize, v_count: usize) -> Vec<Vec<f32>> {
    match model {
        Model::Nurbs(nurbs) => nurbs.get_valid().get_surface_vectors(u_count, v_count),
        _ => vec![vec![0.; 3]; 4],
    }
}

fn get_indices(u_count: usize, v_count: usize) -> Vec<usize> {
    let mut indices = vec![];
    for u in 0..u_count-1 {
        for v in 0..v_count-1 {
            let local_u0_v0 = u * v_count + v;
            let local_u0_v1 = u * v_count + v + 1;
            let local_u1_v0 = (u + 1) * v_count + v;
            let local_u1_v1 = (u + 1) * v_count + v + 1;
            // patch triangle 1
            indices.push(local_u0_v0);
            indices.push(local_u0_v1);
            indices.push(local_u1_v0);
            // patch triangle 2
            indices.push(local_u0_v1);
            indices.push(local_u1_v1);
            indices.push(local_u1_v0);
        }
    }
    indices
}





// #[derive(Clone, Default, Serialize, Deserialize)]
// #[serde(default = "Mesh::default")]
// pub struct Surface {   
//     pub nurbs: Nurbs,
//     pub 
// }

// #[derive(Clone, Serialize, Deserialize)] //#[serde(default = "Control::default")]
// pub enum Shape {
//     Surface(Surface),
// }