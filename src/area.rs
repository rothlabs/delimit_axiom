use super::{Model, DiscreteQuery, log};
use super::mesh::Mesh;
use lyon::path::Path;
// use lyon::path::path::Builder;
// use lyon::path::traits::PathIterator;
use lyon::tessellation::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
}

impl Area { 
    pub fn get_polylines(&self, query: &DiscreteQuery) -> Vec<Vec<f32>> {
        // let mut polylines:Vec<Vec<f32>> = vec![];
        // for part in &self.parts {
        //     polylines.push(part.get_polyline(query));
        //     // match part {
        //     //     Model::Path(m)      => polylines.push(m.get_polyline(tolerance)),
        //     //     Model::Circle(m)    => polylines.push(m.get_polyline(tolerance)),
        //     //     Model::Rectangle(m) => polylines.push(m.get_polyline(tolerance)),
        //     //     _ => ()
        //     // }
        // }
        // polylines
        self.parts.iter().map(|p| p.get_polyline(query)).collect()
    }
    pub fn get_mesh(&self, tolerance: f32) -> Mesh {
        let mut builder = Path::builder();
        for part in &self.parts {
            match part {
                Model::Path(m)      => m.add_parts_to_builder(&mut builder),
                Model::Group(m)     => m.add_parts_to_builder(&mut builder),
                Model::Circle(m)    => m.add_self_to_builder(&mut builder),
                Model::Rectangle(m) => m.add_self_to_builder(&mut builder),
                _ => ()
            }
        }
        let path = builder.build();
        let options = FillOptions::tolerance(tolerance);
        let mut geometry: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
        let mut buffer_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
            let p = vertex.position().to_array();
            [p[0], p[1], 0.]
        });
        let mut tessellator = FillTessellator::new();
        tessellator.tessellate_path(&path, &options, &mut buffer_builder).unwrap(); //.expect("Tessellation failed");
        Mesh {
            vector:    geometry.vertices.into_iter().flatten().collect(),
            triangles: geometry.indices.into_iter().map(|v| v as usize).collect(),
        }
    }
}