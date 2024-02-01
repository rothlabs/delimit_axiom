use super::{Model, DiscreteQuery};
use super::mesh::Mesh;
use lyon::math::{Box2D, Angle, Vector, vector, Point, point};
use lyon::path::polygon::PathEvents;
use lyon::path::traits::{Build, PathBuilder};
use lyon::path::{Path, PathEvent, Position};
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
            //.with_fill_rule(FillRule::EvenOdd)
            //.with_intersections(true)
            //.with_sweep_orientation(Orientation::Horizontal);
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
    let mut builder = lyon::path::Path::builder();//Path::builder();
    let mut endpoint = point(0., 0.); // path.first_endpoint().unwrap_or((point(0., 0.), Attributes::default())).position();
    //let mut end_occurred = false;
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
            PathEvent::Line{from, to} => {
                builder.line_to(to);
            }
            PathEvent::Quadratic { from, ctrl, to } => {
                builder.quadratic_bezier_to(ctrl, to);
            }
            PathEvent::Cubic { from, ctrl1, ctrl2, to }=> {
                builder.cubic_bezier_to(ctrl1, ctrl2, to );
            }
            PathEvent::End { last, first, close } => {
                endpoint.clone_from(&last);
                close_end = close;
            }
        }
    }
    builder.end(close_end);
    builder.build()
}



            // match part {
            //     Model::Group(m) => { 
            //         for path in m.get_paths(){
            //             builder.extend_from_paths(&[path.as_slice()]);
            //         }
            //     },
            //     _ => builder.extend_from_paths(&[part.get_path().as_slice()]),
            // };

// builder.extend_from_paths(&m.get_paths().iter().map(|p| p.as_slice()).collect::<Vec<PathSlice>>().as_slice()),
                    //let path_slices: Vec<lyon::path::PathSlice> = m.get_paths().iter().map(|p| p.as_slice()).collect();
                    //builder.extend_from_paths(&path_slices);
                    //let wow = m.get_paths().as_slice();
                    //builder.extend_from_paths(wow);