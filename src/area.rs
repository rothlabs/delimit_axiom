use super::{Model, DiscreteQuery};
use super::mesh::Mesh;
use lyon::math::point;
use lyon::path::{Path, PathEvent};
use lyon::tessellation::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
}

impl Area { 
    pub fn get_polylines(&self, query: &DiscreteQuery) -> Vec<Vec<f32>> {
        let mut polylines = vec![];
        for part in &self.parts {
            match &part {
                Model::Group(m) => polylines.extend(m.get_polylines(query)),
                _ => polylines.push(part.get_polyline(query)),
            }
        }
        polylines
    }
    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh {
        let mut builder = Path::builder();
        for part in &self.parts {
            for path in part.get_paths(){
                builder.extend_from_paths(&[path.as_slice()]);
            }
        }
        let raw_path = builder.build();
        let path = fuse_path(raw_path);        
        let options = FillOptions::tolerance(query.tolerance);
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

fn fuse_path(path: lyon::path::Path) -> lyon::path::Path{
    let mut builder = lyon::path::Path::builder();
    let mut endpoint = point(0., 0.); // path.first_endpoint().unwrap_or((point(0., 0.), Attributes::default())).position();
    let mut close_end: bool = false;
    let mut open = false;
    for event in path.iter() {
        match event {
            PathEvent::Begin { at } => {
                if open {
                    if endpoint.distance_to(at) > 0.01 {
                        builder.end(close_end);
                        builder.begin(at);
                    }
                }else{
                    builder.begin(at);
                    open = true;
                }
            }
            PathEvent::Line{from:_, to} => {
                builder.line_to(to);
            }
            PathEvent::Quadratic { from:_, ctrl, to } => {
                builder.quadratic_bezier_to(ctrl, to);
            }
            PathEvent::Cubic { from:_, ctrl1, ctrl2, to }=> {
                builder.cubic_bezier_to(ctrl1, ctrl2, to );
            }
            PathEvent::End { last, first:_, close } => {
                endpoint.clone_from(&last);
                close_end = close;
            }
        }
    }
    builder.end(close_end);
    builder.build()
}