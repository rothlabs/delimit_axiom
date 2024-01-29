use super::Model;
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
    pub fn get_polylines(&self, tolerance: f32) -> Vec<Vec<f32>> {
        let mut polylines:Vec<Vec<f32>> = vec![];
        for part in &self.parts {
            match part {
                Model::Path2D(path_2d) => polylines.push(path_2d.get_polyline(tolerance)),
                _ => ()
            }
        }
        polylines
    }
    pub fn get_mesh(&self, tolerance: f32) -> Mesh {
        let mut builder = Path::builder();
        for part in &self.parts {
            match part {
                Model::Path2D(path_2d) => path_2d.add_parts_to_builder(&mut builder),
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