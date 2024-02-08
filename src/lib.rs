mod utils;

//mod vector;
mod group;
mod nurbs;
mod slice;
//mod polyline;
mod mesh;
//mod turtled;
mod sketch;
mod area;
//mod extrusion;
mod revolve;

use utils::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use glam::*;
use group::*;
use nurbs::*;
use slice::*;
//use turtled::*;
use sketch::*;
use area::*;
//use extrusion::*;
use revolve::*;

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
    Point([f32; 3]),
    Curve(Nurbs),
    Facet(Nurbs),
    MoveTo([f32; 2]),
    LineTo([f32; 2]),
    ArcTo(ArcTo),
    Vector(Vec<f32>),
    Sketch(Sketch),
    Area(Area),
    Group(Group),
    Slice(Slice),
    //Turtled(Turtled),
    Circle(Circle),
    Rectangle(Rectangle),
    //Extrusion(Extrusion),
    Revolve(Revolve),
    Close(bool), // TODO: find way to remove bool
}

impl Model {
    pub fn get_shapes(&self) -> Vec<Shape> {
        match self {
            Model::Group(m)   => m.get_shapes(),
            Model::Area(m)    => m.get_shapes(),
            Model::Sketch(m)  => m.get_shapes(),
            Model::Revolve(m) => m.get_shapes(),
            _ => vec![] 
        }
    }
    pub fn get_vec3_or(&self, alt: Vec3) -> Vec3 {
        match self {
            Model::Vector(m) => {
                let vec3 = Vec3::from_slice(m);
                if vec3.length() > 0. {
                    vec3
                } else {
                    alt
                }
            },
            _ => alt,
        }
    }
}

impl Default for Model {
    fn default() -> Self { Model::Vector(vec![0.; 3]) }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Shape {
    Point([f32; 3]),
    Curve(Nurbs),
    Facet(Nurbs),
}

impl Default for Shape {
    fn default() -> Self { Shape::Point([0.; 3]) }
}

impl Shape {
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        match self {
            Shape::Point(m) => Shape::Point(
                mat4.mul_vec4(Vec3::from_slice(m).extend(1.)).truncate().to_array()
            ),
            Shape::Curve(m) => Shape::Curve(m.get_transformed(mat4)),
            Shape::Facet(m) => Shape::Facet(m.get_transformed(mat4)),
        }
    }
}

pub fn get_shapes(parts: &Vec<Model>) -> Vec<Shape> {
    let mut result = vec![];
    for part in parts {
        result.extend(part.get_shapes());
    }
    result
}

pub fn get_curves(parts: &Vec<Model>) -> Vec<Nurbs> {
    let mut result = vec![];
    for part in parts {
        for shape in part.get_shapes() {
            if let Shape::Curve(curve) = shape {
                result.push(curve);
            }
        }
    }
    result
}

pub fn get_points(parts: &Vec<Model>) -> Vec<[f32; 3]> {
    let mut result = vec![];
    for part in parts {
        for shape in part.get_shapes() {
            if let Shape::Point(point) = shape {
                result.push(point);
            }
        }
    }
    result
}

// #[derive(Clone, Default)] 
// pub struct Shape {
//     pub points:   Vec<Vec3>,
//     pub curves:   Vec<Nurbs>,
//     pub surfaces: Vec<Nurbs>,
// }

// impl Shape {
//     pub fn get_transformed(&self, mat4: Mat4) -> Self {
//         self
//     }
// }

#[derive(Clone, Serialize, Deserialize)] 
pub enum Parameter {
    U(f32),
    V(f32),
}

impl Default for Parameter {
    fn default() -> Self { Parameter::U(0.) }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "DiscreteQuery::default")]
pub struct DiscreteQuery {
    pub model:     Model,
    pub count:     usize,
    pub tolerance: f32,   // allowed distance from real model
}

impl DiscreteQuery {
    fn get_valid(self) -> DiscreteQuery {
        let mut count = 4;
        if self.count > 0 { count = self.count.clamp(2, 100); }
        let mut tolerance = 0.1;
        if self.tolerance > 0. { tolerance = self.tolerance.clamp(0.01, 10.); }
        DiscreteQuery {
            model: self.model,
            count,
            tolerance,
        }
    }
}

//pub struct Sample

#[wasm_bindgen]
pub fn enable_panic_messages() {
    set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// macro_rules! console_log {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }
