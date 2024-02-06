use crate::Nurbs;
use super::Model;
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
        let mut shapes = vec![];
        let mut start_point = Model::Point([0.; 3]);//vec![0.; 3];
        for part in &self.parts {
            match part {
                Model::MoveTo(m) => {
                    start_point = Model::Point([m[0], m[1], 0.]);
                    shapes.push(start_point.clone());
                },
                Model::LineTo(m) => {
                    let end_point = Model::Point([m[0], m[1], 0.]);
                    let mut nurbs = Nurbs::default();
                    nurbs.controls = vec![start_point.clone(), end_point.clone()]; // vec![Model::Vector(start_point.clone()), Model::Vector(get_point2(m))];
                    shapes.push(Model::Curve(nurbs));
                    start_point = end_point.clone();
                    shapes.push(end_point);
                },
                // Model::ArcTo(m) => {
                //     let mut nurbs = Nurbs::default();
                //     nurbs.controls = vec![start_point.clone(), m.get_vector_or(start_point)]; 
                //     curves.push(nurbs);
                // },
                _ => (),
            }
        }
        shapes
    }
}



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

// fn get_point(model: &Model) -> Point {
//     match model {
//         Model::Vector(m) => point(m[0], m[1]),
//         _ => point(0., 0.),
//     }
// }

// fn get_point2(model: &Model) -> Vec<f32> {
//     match model {
//         Model::Vector(m) => vec![m[0], m[1], 0.],
//         _ => vec![0.; 3],
//     }
// }

// fn get_vector(model: &Model) -> Vector {
//     match model {
//         Model::Vector(m) => vector(m[0], m[1]),
//         _ => vector(0., 0.),
//     }
// }
