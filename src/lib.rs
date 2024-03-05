mod utils;
mod query;
mod scene;
mod nurbs;
mod spatial;
mod hit;
mod union;
mod reshape;
mod sketch;
mod area;
mod extrude;
mod revolve;
mod grid_pattern;
mod radial_pattern;
mod mirror;

use utils::*;
use nurbs::{curve::*, facet::*};
use spatial::{spatial2::*, spatial3::*};
use hit::{hit2::*, hit3::*};
use union::*;
use reshape::*;
use sketch::*;
use area::*;
use extrude::*;
use revolve::*;
use grid_pattern::*;
use radial_pattern::*;
use mirror::*;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use glam::*;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
    Point([f32; 3]),
    Curve(Curve),
    Facet(Facet),
    Sketch(Sketch),
    Area(Area),
    Reshape(Reshape),
    Arc(Arc),
    Circle(Circle),
    Rectangle(Rectangle),
    Slot(Slot),
    Extrude(Extrude),
    Cuboid(Cuboid),
    Cylinder(Cylinder),
    Revolve(Revolve),
    Union(Union),
    GridPattern(GridPattern),
    RadialPattern(RadialPattern),
    Mirror(Mirror),
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
            Model::Slot(m)      => m.get_shapes(),
            Model::Reshape(m)   => m.get_shapes(),
            Model::Area(m)      => m.get_shapes(),
            Model::Extrude(m)   => m.get_shapes(),
            Model::Cuboid(m)    => m.get_shapes(),
            Model::Cylinder(m)  => m.get_shapes(),
            Model::Revolve(m)   => m.get_shapes(),
            Model::Union(m)     => m.get_shapes(),
            Model::GridPattern(m)   => m.get_shapes(),
            Model::RadialPattern(m) => m.get_shapes(),
            Model::Mirror(m)    => m.get_shapes(),
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
    pub fn get_reshape(&self, mat4: Mat4) -> Self {
        match self {
            Shape::Point(s) => Shape::Point(get_reshaped_point(s, mat4)),
            Shape::Curve(s) => Shape::Curve(s.get_reshape(mat4)),
            Shape::Facet(s) => Shape::Facet(s.get_reshape(mat4)),
        }
    }
    pub fn get_reverse_reshape(&self, mat4: Mat4) -> Self {
        match self {
            Shape::Point(s) => Shape::Point(get_reshaped_point(s, mat4)),
            Shape::Curve(s) => Shape::Curve(s.get_reverse_reshape(mat4)),
            Shape::Facet(s) => Shape::Facet(s.get_reverse_reshape(mat4)),
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

pub fn get_reshapes(parts: &Vec<Shape>, mat4: Mat4) -> Vec<Shape> {
    let mut result = vec![];
    for shape in parts {
        result.push(shape.get_reshape(mat4));
    }
    result
}

// pub fn get_reshapes(parts: &Vec<Model>, mat4: Mat4) -> Vec<Shape> {
//     let mut result = vec![];
//     for part in parts {
//         for shape in part.get_shapes() {
//             result.push(shape.get_reshape(mat4));
//         }
//     }
//     result
// }

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

pub fn get_grouped_curves(parts: &Vec<Model>) -> Vec<Vec<CurveShape>> {
    let mut curves = vec![];
    for part in parts {
        let mut group = vec![];
        for shape in part.get_shapes() {
            match shape {
                Shape::Curve(s) => group.push(s),
                _ => (),
            }
        }
        curves.push(group);
    }
    curves
}

pub fn get_grouped_facets(parts: &Vec<Model>) -> Vec<Vec<FacetShape>> {
    let mut facets = vec![];
    for part in parts {
        let mut group = vec![];
        for shape in part.get_shapes() {
            match shape {
                Shape::Facet(s) => group.push(s),
                _ => (),
            }
        }
        facets.push(group);
    }
    facets
}

pub fn get_grouped_curves_and_facets(parts: &Vec<Model>) -> (Vec<CurveShape>, Vec<FacetShape>, Vec<Vec<CurveShape>>, Vec<Vec<FacetShape>>) {
    let mut curves = vec![];
    let mut facets = vec![];
    let mut curve_groups = vec![];
    let mut facet_groups = vec![];
    for part in parts {
        let mut curve_group = vec![];
        let mut facet_group = vec![];
        for shape in part.get_shapes() {
            match shape {
                Shape::Curve(s) => curve_group.push(s),
                Shape::Facet(s) => facet_group.push(s),
                _ => (),
            }
        }
        curve_groups.push(curve_group.clone());
        facet_groups.push(facet_group.clone());
        curves.extend(curve_group);
        facets.extend(facet_group);
    }
    (curves, facets, curve_groups, facet_groups)
}

pub fn get_reshaped_point(point: &Vec3, mat4: Mat4) -> Vec3 { // [f32; 3] {
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

pub fn get_line_intersection2(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    // let t = ((p1.x - p3.x)*(p3.y - p4.y) - (p1.y - p3.y)*(p3.x - p4.x)) 
    //     / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    // let x = p1.x + t*(p2.x - p1.x);
    // let y = p1.y + t*(p2.y - p1.y);
    let u = - ((p1.x - p2.x)*(p1.y - p3.y) - (p1.y - p2.y)*(p1.x - p3.x))
        / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    let x = p3.x + u*(p4.x - p3.x);
    let y = p3.y + u*(p4.y - p3.y);
    if x.is_nan() || y.is_nan() {
        return None;
    }
    Some(vec2(x, y))
}

fn get_line_intersection3(p1: Vec3, d1: Vec3, p2: Vec3, d2: Vec3) -> Vec3 {
    let v = p1 - p2;
    let a = d1.dot(d1);
    let b = d1.dot(d2);
    let c = d2.dot(d2);
    let d = d1.dot(v);
    let e = d2.dot(v);

    let denom = a * c - b * b;
    let t = (b * e - c * d) / denom;
    let s = (a * e - b * d) / denom;

    let p_closest = p1 + t * d1;
    let q_closest = p2 + s * d2;

    (p_closest + q_closest) / 2.//(p_closest, q_closest)
}

pub fn get_vector_hash(vecf32: &Vec<f32>) -> u64 {
    let veci32: Vec<u64> = vecf32.iter().enumerate().map(|(i, v)| i as u64 * (v * 10000.).floor() as u64).collect();
    veci32.iter().sum()
    // let mut hasher = DefaultHasher::new();
    // veci32.hash(&mut hasher);
    // hasher.finish()
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
