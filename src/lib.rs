mod utils;
mod query;
mod result;
mod nurbs;
mod spatial;
mod group;
mod sketch;
mod area;
mod revolve;
mod union;

use utils::*;
use nurbs::{curve::*, facet::*};
use spatial::{spatial2::*, spatial3::*};
use group::*;
use sketch::*;
use area::*;
use revolve::*;
use union::*;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use glam::*;

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
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
            Model::Point(m)     => vec![Shape::Point(Vec3::from_array(*m))],
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
        }
    }
}

impl Default for Model {
    fn default() -> Self { 
        Model::Point([0.; 3]) 
    }
}

#[derive(Clone)] 
pub enum Shape {
    Point(Vec3),
    Curve(CurveShape),
    Facet(FacetShape),
}

impl Shape {
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        match self {
            Shape::Point(s) => Shape::Point(get_transformed_point(s, mat4)),
            Shape::Curve(s) => Shape::Curve(s.get_transformed(mat4)),
            Shape::Facet(s) => Shape::Facet(s.get_transformed(mat4)),
        }
    }
}

impl Default for Shape {
    fn default() -> Self { 
        Shape::Point(Vec3::ZERO) 
    }
}

pub fn get_shapes(parts: &Vec<Model>) -> Vec<Shape> {
    let mut result = vec![];
    for part in parts {
        result.extend(part.get_shapes());
    }
    result
}

pub fn get_points(parts: &Vec<Model>) -> Vec<Vec3> {
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

pub fn get_facets(parts: &Vec<Model>) -> Vec<FacetShape> {
    let mut result = vec![];
    for part in parts {
        for shape in part.get_shapes() {
            if let Shape::Facet(facet) = shape {
                result.push(facet);
            }
        }
    }
    result
}

pub fn get_curves_and_facets(parts: &Vec<Model>) -> (Vec<CurveShape>, Vec<FacetShape>) {
    let mut curves = vec![];
    let mut facets = vec![];
    for part in parts {
        for shape in part.get_shapes() {
            match shape {
                Shape::Curve(s) => curves.push(s),
                Shape::Facet(s) => facets.push(s),
                _ => (),
            }
        }
    }
    (curves, facets)
}

pub fn get_transformed_point(point: &Vec3, mat4: Mat4) -> Vec3 { // [f32; 3] {
    mat4.mul_vec4(point.extend(1.)).truncate() //mat4.mul_vec4(Vec3::from_slice(point).extend(1.)).truncate().to_array()
}

pub fn get_vec3_or(slice: &[f32; 3], alt: Vec3) -> Vec3 {
    let vec3 = Vec3::from_slice(slice);
    if vec3.length() > 0. {
        vec3
    } else {
        alt
    }
}

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
