use super::{Model, DiscreteQuery};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub fn get_mesh_from_faces(parts: Vec<Model>, query: &DiscreteQuery) -> Mesh {
    let mut vector: Vec<f32> = vec![];
    let mut trivec: Vec<usize> = vec![];
    let mut offset = 0; 
    for part in &parts {
        let mesh = match &part {
            Model::Area(m) =>  m.get_mesh(query),
            Model::Nurbs(m) => m.get_mesh(query),
            _ => Mesh::default(),
        };
        vector.extend(&mesh.vector);
        trivec.extend::<Vec<usize>>(mesh.triangles.iter().map(|v| v + offset).collect::<Vec<usize>>());
        offset += mesh.vector.len() / 3;
    }
    Mesh {
        vector,
        triangles: trivec,
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vector:    Vec<f32>, 
    pub triangles: Vec<usize>,
}

#[wasm_bindgen]
pub fn get_mesh(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let mesh = match &query.model {
        Model::Area(m)      => m.get_mesh(&query),
        Model::Extrusion(m) => m.get_mesh(&query),
        Model::Revolve(m)   => m.get_mesh(&query),
        _ => Mesh {
            vector: vec![0.; 9],
            triangles: vec![0, 1, 2],
        }
    };
    Ok(serde_wasm_bindgen::to_value(&mesh)?)
}

#[wasm_bindgen]
pub fn get_mesh_vector(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let vector = match &query.model {
        Model::Nurbs(nurbs) => nurbs.get_mesh_vector(&query),
        _ => vec![0.; 12],
    };
    Ok(serde_wasm_bindgen::to_value(&vector)?)
}

#[wasm_bindgen]
pub fn get_triangles(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?;
    let query = query.get_valid();
    let vector = get_trivec(&query); // query.u_count, query.v_count
    Ok(serde_wasm_bindgen::to_value(&vector)?)
}

pub fn get_trivec(query: &DiscreteQuery) -> Vec<usize>{ // u_count: usize, v_count: usize
    let &DiscreteQuery {u_count, v_count, ..} = query;
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



// fn clamp_counts(u_count: usize, v_count: usize) -> [usize; 2] {
//     [u_count.clamp(2, 1000), v_count.clamp(2, 1000)]
// }

// #[derive(Default, Serialize, Deserialize)]
// #[serde(default = "MeshQuery::default")]
// pub struct MeshQuery {
//     model: Model,
//     pub tolerance: f32,
//     pub count: usize,
//     u_count: usize,
//     v_count: usize,
// }


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