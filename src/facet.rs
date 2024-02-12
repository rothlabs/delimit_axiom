use std::f32::consts::PI;

use crate::mesh::Mesh;
//use crate::BoundaryV;
use crate::curve::Curve;
use crate::CurveShape;
use super::{Model, Shape, Parameter, DiscreteQuery, get_curves, log};
use glam::*;
use serde::{Deserialize, Serialize};
use lyon::tessellation::*;
use lyon::geom::{Box2D, Point};
use lyon::path::Winding;
//use rayon::prelude::*;

// ((a % b) + b) % b)  ->  a modulo b

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

//static default_boundary: BoundaryV = BoundaryV::default();

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Facet::default")]
pub struct Facet {
    pub controls:    Vec<Model>,
    pub boundaries:  Vec<Model>,
    pub order:       usize,       // order = polynomial_degree + 1
    pub knots:       Vec<f32>,    // knot_count = order + control_count
    pub weights:     Vec<f32>,    // weight_count = control_count
    pub reversed:    bool,
}

impl Facet {
    pub fn get_shapes(&self) -> Vec<Shape> {
        vec![Shape::Facet(FacetShape{
            controls:   get_curves(&self.controls),
            boundaries: get_curves(&self.boundaries),
            knots:      self.knots.clone(),
            weights:    self.weights.clone(),
            order:      self.order,
            reversed:   self.reversed,
        })]
    }
}

#[derive(Clone)]
pub struct FacetShape {
    pub controls:    Vec<CurveShape>,
    pub boundaries:  Vec<CurveShape>,
    pub order:       usize,       // order = polynomial_degree + 1
    pub knots:       Vec<f32>,    // knot_count = order + control_count
    pub weights:     Vec<f32>,    // weight_count = control_count
    pub reversed:    bool,
}

impl Default for FacetShape {
    fn default() -> Self {
        FacetShape {
            controls: vec![],
            boundaries: vec![],
            knots: vec![],
            weights: vec![],
            order: 2,
            reversed: false,
        }
    }
}

impl FacetShape { // impl<T: Default + IntoIterator<Item=f32>> FacetShape<T> {
    pub fn get_reversed_and_transformed(&self, mat4: Mat4) -> FacetShape {
        let mut facet = FacetShape {
            order: self.order,
            knots: self.knots.clone(),
            weights: self.weights.clone(),
            controls: vec![],
            boundaries: self.boundaries.clone(),
            reversed: true,
        };
        for control in self.controls.iter().rev() {
            facet.controls.push(control.get_transformed(mat4));
        }
        facet
    }

    pub fn get_transformed(&self, mat4: Mat4) -> FacetShape {
        let mut facet = FacetShape {
            order: self.order,
            knots: self.knots.clone(),
            weights: self.weights.clone(),
            controls: vec![],
            boundaries: self.boundaries.clone(),
            reversed: false,
        };
        for control in &self.controls {
            facet.controls.push(control.get_transformed(mat4));
        }
        facet
    }

    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh { 
        log("get mesh!! 0");
        let facet = self.get_valid();
        log("valid done");
        let mut u_count = 0;
        for curve in &facet.controls {
            let sample_count = curve.get_sample_count(query.count);
            console_log!("curve sample count done {}", sample_count);
            if u_count < sample_count { u_count = sample_count; } 
        }
        let v_count = facet.get_sample_count(query.count);
        console_log!("u_count {}", u_count);
        console_log!("v_count {}", v_count);
        let mut builder = lyon::path::Path::builder();
        if facet.boundaries.is_empty() {
            builder.add_rectangle(&Box2D{min:Point::new(0., 0.), max:Point::new(1., 1.)}, Winding::Positive);
        }
        for ui in 0..u_count {
            let u = ui as f32 / (u_count-1) as f32;
            builder.add_rectangle(&Box2D{min:Point::new(u, 0.), max:Point::new(u, 1.)}, Winding::Positive);
        }
        for vi in 0..v_count {
            let v = vi as f32 / (v_count-1) as f32;
            builder.add_rectangle(&Box2D{min:Point::new(0., v), max:Point::new(1., v)}, Winding::Positive);
        }
        let mut open_loop = false;
        let mut start_point = lyon::geom::Point::default();
        for boundary in &facet.boundaries { 
            for p in boundary.get_polyline(query).chunks(3) {
                let mut y = p[1];
                if facet.reversed {y = 1.-y;}
                let point = lyon::geom::Point::new(p[0], y);
                if open_loop {
                    if start_point.distance_to(point) > f32::EPSILON { // f32::EPSILON*1000.
                        builder.line_to(point);
                    }else {
                        builder.end(true);
                        open_loop = false;
                    }
                }else{
                    builder.begin(point);
                    start_point = point;
                    open_loop = true;
                }
            }
        }
        let path = builder.build();
        let options = FillOptions::default(); //tolerance(query.tolerance);
        let mut geometry: VertexBuffers<[f32; 2], usize> = VertexBuffers::new();
        let mut buffer_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| vertex.position().to_array());
        let mut tessellator = FillTessellator::new();
        tessellator.tessellate_path(&path, &options, &mut buffer_builder).expect("Tessellation failed");
        let mut vector = vec![];
        for [u, v] in geometry.vertices.into_iter(){
            vector.extend(facet.get_vector_at_uv(u, v));
        }
        Mesh {
            vector, //:    geometry.vertices.into_iter().flatten().collect(),
            triangles: geometry.indices, 
        }
    }

    // pub fn get_polyline_at_parameter(&self, t: &Parameter, count: usize) -> Vec<Vec<f32>> {
    //     let facet = self.get_valid();
    //     match t {
    //         Parameter::U(u) => facet.get_polyline_at_u(*u, count),
    //         Parameter::V(v) => facet.get_polyline_at_v(*v, count),
    //     }
    // }

    fn get_sample_count(&self, count: usize) -> usize { 
        let mul = self.controls.len()-1;
        self.controls.len() + count * (self.order - 2) * mul
    }

    pub fn get_vector_at_uv(&self, u: f32, v: f32) -> Vec<f32> {
        let basis = self.get_rational_basis_at_v(v);
        let mut vector = vec![];
        if ! self.controls.is_empty() {
            for component_index in 0..3 { 
                vector.push(
                    (0..self.controls.len())
                        .map(|i| self.controls[i].get_vector_at_u(u)[component_index] * basis[i]).sum()
                );
            }
        }
        vector
    }

    fn get_rational_basis_at_v(&self, v: f32) -> Vec<f32> {
        let basis = self.get_basis_at_v(v);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    fn get_basis_at_v(&self, normal_v: f32) -> Vec<f32> {
        let t = *self.knots.last().unwrap_or(&0.) * normal_v; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_v(t);
        for degree in 1..self.order {
            for i0 in 0..self.controls.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] != 0. {
                    f = (t - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
                }
                if basis[i1] != 0. {
                    g = (self.knots[degree + i1] - t) / (self.knots[degree + i1] - self.knots[i1])
                }
                basis[i0] = f * basis[i0] + g * basis[i1];
            }
        }
        if normal_v == 1. { 
            basis[self.controls.len() - 1] = 1.; // last control edge case
        }
        basis
    }

    fn get_basis_of_degree_0_at_v(&self, v: f32) -> Vec<f32> {
        self.knots.windows(2)
            .map(|knots| {
                if v >= knots[0] && v < knots[1] {
                    1.
                } else {
                    0.
                }
            }).collect()
    }

    fn get_valid(&self) -> FacetShape {
        // let order = self.get_valid_order();
        // console_log!("order {}", order);
        // let knots = self.get_valid_knots();
        // console_log!("knots {}", knots.len());
        // let weights = self.get_valid_weights();
        // console_log!("weights {}", weights.len());
        // let controls: Vec<Curve> = self.controls.iter().map(|c| c.get_valid()).collect();
        // console_log!("controls {}", controls.len());
        FacetShape {
            order: self.get_valid_order(),
            knots: self.get_valid_knots(),
            weights: self.get_valid_weights(),
            controls: self.controls.iter().map(|c| c.get_valid()).collect(), // self.controls.clone(), //
            boundaries: self.boundaries.clone(),
            reversed: self.reversed,
        }
    }

    fn get_valid_order(&self) -> usize {
        self.order.min(self.controls.len()).max(2)
    }

    fn get_valid_weights(&self) -> Vec<f32> {
        if self.weights.len() == self.controls.len() {
            self.weights.clone()
        } else {
            vec![1.; self.controls.len()]
        }
    }

    fn get_valid_knots(&self) -> Vec<f32> {
        if self.knots.len() == self.controls.len() + self.get_valid_order() {
            self.knots.clone()
        } else {
            self.get_open_knots()
        }
    }

    fn get_open_knots(&self) -> Vec<f32> {
        let order = self.get_valid_order();
        let repeats = order - 1; // knot multiplicity = order for ends of knot vector
        let max_knot = self.controls.len() + order - (repeats * 2) - 1;
        let mut knots = vec![0_f32; repeats];
        knots.extend((0..=max_knot).map(|k| k as f32));
        knots.extend(vec![max_knot as f32; repeats]);
        knots
    }
}



// // visual tests
// impl FacetShape {
//     // for examining the "basis functions" as pictured on wikipedia
//     pub fn get_basis_plot_vectors(&self, control_index: usize, count: usize) -> Vec<Vec<f32>> {
//         let max_t = *self.knots.last().unwrap_or(&0.); // .unwrap_throw("") to javascript client
//         (0..count)
//             .map(|t| {
//                 let x = (max_t / (count - 1) as f32) * t as f32;
//                 vec![x, self.get_basis_at_t(x)[control_index], 0.]
//             })
//             .collect()
//     }
// }