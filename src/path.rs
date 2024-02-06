use crate::{DiscreteQuery, Group, Nurbs};

use super::{Model, log};
use lyon::geom::euclid::Point2D;
use lyon::math::{Box2D, Angle, Vector, vector, Point, point};
use lyon::path::{ArcFlags, Event, PathEvent, Winding}; // PathBuffer
use lyon::path::builder::BorderRadii;
use lyon::path::traits::{PathIterator, SvgPathBuilder};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Sketch::default")]
pub struct Sketch {
    pub parts: Vec<Model>,
    //pub reverse: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "ArcTo::default")]
pub struct ArcTo {
    pub angle:  f32,
    pub radius: f32,
    //pub radii: Box<Model>,
}

impl Sketch { 
    pub fn get_shapes(&self) -> Vec<Model> {
        let mut curves = vec![];
        let mut start_point = Model::Vector(vec![0.; 3]);//vec![0.; 3];
        for part in &self.parts {
            match part {
                Model::MoveTo(m) => start_point = m.get_vector_or(start_point),//get_point2(m),
                Model::LineTo(m) => {
                    let mut nurbs = Nurbs::default();
                    nurbs.controls = vec![start_point.clone(), m.get_vector()]; // vec![Model::Vector(start_point.clone()), Model::Vector(get_point2(m))];
                    curves.push(nurbs);
                },
                Model::ArcTo(m) => {
                    let mut nurbs = Nurbs::default();
                    nurbs.controls = vec![Model::Vector(start_point.clone()), Model::Vector(get_point2(&m.to))];
                    curves.push(nurbs);
                },
                _ => (),
            }
        }
        curves
    }
}

// macro_rules! console_log {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center: Box<Model>, 
    pub radius: f32,
}

// impl Circle {
//     pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
//         get_polyline(self.get_path(), tolerance)
//     }
//     pub fn get_path(&self) -> lyon::path::Path {
//         let center = get_point(&*self.center);
//         let mut builder = lyon::path::Path::builder();
//         builder.add_circle(center, self.radius, Winding::Positive); 
//         builder.build()
//     }
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    Box<Model>, 
    pub max:    Box<Model>, 
    pub radius: f32,
}

// impl Rectangle {
//     pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
//         get_polyline(self.get_path(), tolerance)
//     }
//     pub fn get_path(&self) -> lyon::path::Path { 
//         let min = get_point(&*self.min);
//         let max = get_point(&*self.max);
//         let mut builder = lyon::path::Path::builder();
//         builder.add_rounded_rectangle(
//             &Box2D {min, max},
//             &BorderRadii {
//                 top_left:     self.radius,
//                 top_right:    self.radius,
//                 bottom_left:  self.radius,
//                 bottom_right: self.radius,
//             },
//             Winding::Positive,
//         );
//         builder.build()
//     }
// }

fn get_point(model: &Model) -> Point {
    match model {
        Model::Vector(m) => point(m[0], m[1]),
        _ => point(0., 0.),
    }
}

fn get_point2(model: &Model) -> Vec<f32> {
    match model {
        Model::Vector(m) => vec![m[0], m[1], 0.],
        _ => vec![0.; 3],
    }
}

fn get_vector(model: &Model) -> Vector {
    match model {
        Model::Vector(m) => vector(m[0], m[1]),
        _ => vector(0., 0.),
    }
}
