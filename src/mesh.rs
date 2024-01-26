use super::Model;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "MeshQuery::default")]
struct MeshQuery {
    u_count: usize,
    v_count: usize,
    model: Model,
}

#[derive(Serialize, Deserialize)]
struct Mesh {
    vector: Vec<f32>, 
    triangles: Vec<usize>,
}

#[wasm_bindgen]
pub fn get_mesh(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: MeshQuery = serde_wasm_bindgen::from_value(val)?;
    let u_count = queried.u_count.clamp(2, 1000);
    let v_count = queried.v_count.clamp(2, 1000); 
    let result = Mesh {
        vector: get_mesh_vector(queried.model, u_count, v_count),
        triangles: get_triangles(u_count, v_count),
    };
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

fn get_mesh_vector(model: Model, u_count: usize, v_count: usize) -> Vec<f32> {
    match model {
        Model::Nurbs(nurbs) => nurbs.get_valid().get_mesh_vector(u_count, v_count),
        _ => vec![0.; 12],
    }
}

fn get_triangles(u_count: usize, v_count: usize) -> Vec<usize> {
    let mut triangles = vec![];
    for u in 0..u_count-1 {
        for v in 0..v_count-1 {
            let local_u0_v0 = u * v_count + v;
            let local_u0_v1 = u * v_count + v + 1;
            let local_u1_v0 = (u + 1) * v_count + v;
            let local_u1_v1 = (u + 1) * v_count + v + 1;
            // patch triangle 1
            triangles.push(local_u0_v0);
            triangles.push(local_u0_v1);
            triangles.push(local_u1_v0);
            // patch triangle 2
            triangles.push(local_u0_v1);
            triangles.push(local_u1_v1);
            triangles.push(local_u1_v0);
        }
    }
    triangles
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