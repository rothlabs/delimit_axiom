use super::Model;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "MeshQuery::default")]
struct MeshQuery {
    model: Model,
    u_count: usize,
    v_count: usize,
    tolerance: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub vector:    Vec<f32>, 
    pub triangles: Vec<usize>,
}

#[wasm_bindgen]
pub fn get_mesh(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: MeshQuery = serde_wasm_bindgen::from_value(val)?;
    let mut tolerance = 0.1;
    if queried.tolerance > 0. { tolerance = queried.tolerance.clamp(0.01, 10.); }
    let mesh = match queried.model {
        Model::Area(area)       =>  area.get_mesh(tolerance),
        Model::Extrusion(extru) => extru.get_mesh(tolerance),
        _ => Mesh {
            vector: vec![0.; 9],
            triangles: vec![0, 1, 2],
        }
    };
    Ok(serde_wasm_bindgen::to_value(&mesh)?)
}

#[wasm_bindgen]
pub fn get_mesh_vector(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: MeshQuery = serde_wasm_bindgen::from_value(val)?;
    let [u_count, v_count] = clamp_counts(queried.u_count, queried.v_count);
    let vector = match queried.model {
        Model::Nurbs(nurbs) => nurbs.get_mesh_vector(u_count, v_count),
        _ => vec![0.; 12],
    };
    Ok(serde_wasm_bindgen::to_value(&vector)?)
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "TriangleQuery::default")]
struct TriangleQuery {
    u_count: usize,
    v_count: usize,
}

#[wasm_bindgen]
pub fn get_triangles(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: TriangleQuery = serde_wasm_bindgen::from_value(val)?;
    let [u_count, v_count] = clamp_counts(queried.u_count, queried.v_count); 
    let vector = get_trivec(u_count, v_count);
    Ok(serde_wasm_bindgen::to_value(&vector)?)
}

pub fn get_trivec(u_count: usize, v_count: usize) -> Vec<usize>{
    get_trivec_with_offset(u_count, v_count, 0)
}

pub fn get_trivec_with_offset(u_count: usize, v_count: usize, offset: usize) -> Vec<usize>{
    let mut vector = vec![];
    for u in 0..u_count-1 {
        for v in 0..v_count-1 {
            let local_u0_v0 = u * v_count + v + offset;
            let local_u0_v1 = u * v_count + v + 1 + offset;
            let local_u1_v0 = (u + 1) * v_count + v + offset;
            let local_u1_v1 = (u + 1) * v_count + v + 1 + offset;
            // patch triangle 1
            vector.push(local_u0_v0);
            vector.push(local_u0_v1);
            vector.push(local_u1_v0);
            // patch triangle 2
            vector.push(local_u0_v1);
            vector.push(local_u1_v1);
            vector.push(local_u1_v0);
        }
    }
    vector
}

fn clamp_counts(u_count: usize, v_count: usize) -> [usize; 2] {
    [u_count.clamp(2, 1000), v_count.clamp(2, 1000)]
}


    // let path = builder.build();
    // let options = FillOptions::tolerance(tolerance);
    // let mut geometry: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
    // let mut buffer_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
    //     let p = vertex.position().to_array();
    //     [p[0], p[1], 0.]
    // });
    // let mut tessellator = FillTessellator::new();
    // tessellator.tessellate_path(&path, &options, &mut buffer_builder).unwrap(); //.expect("Tessellation failed");
    // let mesh = Mesh {
    //     vector: geometry.vertices.into_iter().flatten().collect(),
    //     triangles: geometry.indices,
    // };

    // let options = FillOptions {
    //     tolerance,
    //     fill_rule: FillRule::EvenOdd,
    //     sweep_orientation: Orientation::Horizontal,
    //     handle_intersections: true,
    // };



        //let point = vertex.position();
        //Vec3::new(point.x, point.y, 0.)

// let [u_count, _] = clamp_counts(queried.u_count, queried.v_count); 
// let polyline = match queried.model {
//     Model::Turtled(turtled) => turtled.get_polyline(u_count),
//     _ => vec![0.; 12],
// };
// // let polyline_iter = PolylineIterator::new(polyline);
// let polygon = Polygon {
//     points: &[point(0.,1.), point(1.,1.), point(2.,5.)],
//     closed: true,
// };
// let mut builder = Path::builder();
// builder.add_polygon(polygon);
// let path = builder.build();








// #[derive(Serialize, Deserialize)]
// struct Mesh {
//     vector: Vec<f32>, 
//     triangles: Vec<usize>,
// }



// // #[derive(Serialize, Deserialize)]
// // struct Mesh {
// //     vector: Vec<f32>, 
// //     triangles: Vec<usize>,
// // }

// #[wasm_bindgen]
// pub fn get_mesh_vector(val: JsValue) -> Result<JsValue, JsValue> {
//     let queried: MeshQuery = serde_wasm_bindgen::from_value(val)?;
//     let u_count = queried.u_count.clamp(2, 1000);
//     let v_count = queried.v_count.clamp(2, 1000); 
//     let result = Mesh {
//         vector: get_mesh_vector(queried.model, u_count, v_count),
//         triangles: get_triangles(u_count, v_count),
//     };
//     Ok(serde_wasm_bindgen::to_value(&result)?)
// }
// fn get_mesh_vector(model: Model, u_count: usize, v_count: usize) -> Vec<f32> {
//     match model {
//         Model::Nurbs(nurbs) => nurbs.get_mesh_vector(u_count, v_count),
//         _ => vec![0.; 12],
//     }
// }



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