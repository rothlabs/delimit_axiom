mod utils;

mod vector;
mod group;
mod nurbs;
mod slice;
mod polyline;
mod mesh;
mod turtled;
mod path;
mod area;
mod extrusion;
mod revolve;

use utils::*;
use serde::{Deserialize, Serialize};
use vector::get_transformed_vector;
use wasm_bindgen::prelude::*;
use glam::*;
use group::*;
use nurbs::*;
use slice::*;
use turtled::*;
use path::*;
use area::*;
use extrusion::*;
use revolve::*;

#[derive(Clone, Serialize, Deserialize)] 
pub enum Model {
    Vector(Vec<f32>),
    
    //Point(Vec<f32>),
    Nurbs(Nurbs),
    //Curve(Nurbs),
    //Surface(Nurbs),
    Sketch(Sketch),
    Group(Group),
    Slice(Slice),
    Turtled(Turtled),
    Circle(Circle),
    Rectangle(Rectangle),
    Area(Area),
    Extrusion(Extrusion),
    Revolve(Revolve),
    
    MoveTo(Box<Model>),
    LineTo(Box<Model>),
    ArcTo(ArcTo),
    Close(bool), // TODO: find way to remove bool
}

impl Model {
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        match self {
            Model::Vector(m) => Model::Vector(get_transformed_vector(m, mat4)),
            Model::Nurbs(m)  => Model::Nurbs(m.get_transformed(mat4)),
            _ => self.clone()
        }
    }
    pub fn get_shapes(&self) -> Vec<Model> {
        match self {
            Model::Vector(m)  => vec![Model::Vector(m.clone())],
            Model::Nurbs(m)   => vec![Model::Nurbs(m.clone())],
            //Model::Path(m)    => m.get_shapes(),
            Model::Revolve(m) => m.get_shapes(),
            //Model::Group(m)  => 
            //Model::Path(m)   => 
            _ => vec![] 
        }
    }
    pub fn get_vector_or(&self, alt: Model) -> Model {
        match self {
            Model::Vector(m) => self.clone(),
            _ => alt,
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

pub fn get_points_and_curves(parts: &Vec<Model>) -> Vec<Model> {
    let mut result = vec![];
    for part in parts {
        for shape in part.get_shapes(){
            match shape {
                Model::Vector(m) => result.push(part.clone()),
                Model::Nurbs(m)  => result.push(part.clone()),
                _ => ()
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
    pub tolerance: f32,   // allowed distance from real model
    pub count:     usize, // quantity of points from the model (when tolerance is not implemented)
    pub u_count:   usize, // for surfaces
    pub v_count:   usize, // for surfaces
}

impl DiscreteQuery {
    fn get_valid(self) -> DiscreteQuery {
        let mut tolerance = 0.1;
        if self.tolerance > 0. { tolerance = self.tolerance.clamp(0.01, 10.); }
        let mut count = 80;
        if self.count > 0 { count = self.count.clamp(2, 1000); }
        let mut u_count = 25;
        if self.u_count > 0 { u_count = self.u_count.clamp(2, 1000); }
        let mut v_count = 25;
        if self.v_count > 0 { v_count = self.v_count.clamp(2, 1000); }
        DiscreteQuery {
            model: self.model,
            tolerance,
            count,
            u_count,
            v_count,
        }
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
