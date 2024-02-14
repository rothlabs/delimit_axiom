mod utils;

mod spatial_map;
mod group;
mod curve;
mod facet;
mod mesh;
mod sketch;
mod area;
mod revolve;
mod union;

use utils::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use glam::*;
use spatial_map::*;
use group::*;
use curve::*;
use facet::*;
use sketch::*;
use area::*;
use revolve::*;
use union::*;

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
    Vector3([f32; 3]),
    Point([f32; 3]),
    Curve(Curve),
    Facet(Facet),
    Sketch(Sketch),
    Area(Area),
    Group(Group),
    Arc(Arc),
    Circle(Arc),
    Rectangle(Rectangle),
    Revolve(Revolve),
    Union(Union),
}

impl Model {
    pub fn get_shapes(&self) -> Vec<Shape> {
        match self {
            Model::Point(m)     => vec![Shape::Point(*m)],
            Model::Curve(m)     => m.get_shapes(),
            Model::Facet(m)     => m.get_shapes(),
            Model::Sketch(m)    => m.get_shapes(),
            Model::Arc(m)       => m.get_shapes(),
            Model::Circle(m)    => m.get_shapes(),
            Model::Rectangle(m) => m.get_shapes(),
            Model::Group(m)     => m.get_shapes(),
            Model::Area(m)      => m.get_shapes(),
            Model::Revolve(m)   => m.get_shapes(),
            Model::Union(m)     => m.get_shapes(),
            _ => vec![] 
        }
    }
    // pub fn get_transformed(&self, mat4: Mat4) -> Self {
    //     let mut shapes = vec![];
    //     for shape in self.get_shapes()
    // }
    // pub fn get_vec3_or(&self, alt: Vec3) -> Vec3 {
    //     match self {
    //         Model::Vector3(m) => {
    //             let vec3 = Vec3::from_slice(m);
    //             if vec3.length() > 0. {
    //                 vec3
    //             } else {
    //                 alt
    //             }
    //         },
    //         _ => alt,
    //     }
    // }
}

impl Default for Model {
    fn default() -> Self { Model::Vector3([0.; 3]) }
}

pub fn get_vec3_or(slice: &[f32; 3], alt: Vec3) -> Vec3 {
    let vec3 = Vec3::from_slice(slice);
    if vec3.length() > 0. {
        vec3
    } else {
        alt
    }
}

#[derive(Clone)] // Serialize, Deserialize
pub enum Shape {
    Point([f32; 3]),
    Curve(CurveShape),
    Facet(FacetShape),
}

impl Default for Shape {
    fn default() -> Self { Shape::Point([0.; 3]) }
}

impl Shape {
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        match self {
            Shape::Point(point) => Shape::Point(get_transformed_point(point, mat4)),
            Shape::Curve(m) => Shape::Curve(m.get_transformed(mat4)),
            Shape::Facet(m) => Shape::Facet(m.get_transformed(mat4)),
        }
    }
}

pub fn get_transformed_point(point: &[f32; 3], mat4: Mat4) -> [f32; 3] {
    mat4.mul_vec4(Vec3::from_slice(point).extend(1.)).truncate().to_array()
}

pub fn get_shapes(parts: &Vec<Model>) -> Vec<Shape> {
    let mut result = vec![];
    for part in parts {
        result.extend(part.get_shapes());
    }
    result
}

pub fn get_curves(parts: &Vec<Model>) -> Vec<CurveShape> {
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

// #[derive(Clone, Serialize, Deserialize)]
// pub enum Boundary {
//     V(BoundaryV), // U and angle
//     Curve(Nurbs),
// }

// impl Default for Boundary {
//     fn default() -> Self { Boundary::V(BoundaryV::default()) }
// }

// #[derive(Clone, Default, Serialize, Deserialize)]
// pub struct BoundaryV {
//     v: f32,
//     angle: f32,
// }


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
        let mut count = 8;
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
