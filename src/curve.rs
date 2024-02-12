use std::f32::consts::PI;

use crate::{Model, Shape, Parameter, DiscreteQuery, get_points, get_transformed_point, log};
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
#[serde(default = "Curve::default")]
pub struct Curve {
    pub controls: Vec<Model>,
    pub knots:    Vec<f32>,    // knot_count = order + control_count
    pub weights:  Vec<f32>,    // weight_count = control_count
    pub order:    usize,       // order = polynomial_degree + 1
    pub min:      f32,
    pub max:      f32,
}

impl Curve {
    pub fn get_shapes(&self) -> Vec<Shape> {
        
        vec![Shape::Curve(CurveShape{
            controls: get_points(&self.controls),
            knots: self.knots.clone(),
            weights: self.weights.clone(),
            order: self.order,
            min: self.min,
            max: self.max,
        })]
    }
}

#[derive(Clone)]
pub struct CurveShape {
    pub controls: Vec<[f32; 3]>,
    pub knots:    Vec<f32>,    // knot_count = order + control_count
    pub weights:  Vec<f32>,    // weight_count = control_count
    pub order:    usize,       // order = polynomial_degree + 1
    pub min:      f32,
    pub max:      f32,
}

impl Default for CurveShape {
    fn default() -> Self {
        CurveShape {
            controls: vec![],
            knots: vec![],
            weights: vec![],
            order: 2,
            min: 0.,
            max: 1.,
        }
    }
}

impl CurveShape { // impl<T: Default + IntoIterator<Item=f32>> Curve<T> {
    pub fn get_transformed(&self, mat4: Mat4) -> CurveShape {
        let mut curve = CurveShape {
            controls: vec![],
            order: self.order,
            knots: self.knots.clone(),
            weights: self.weights.clone(),
            min: self.min,
            max: self.max,
        };
        for point in &self.controls {
            curve.controls.push(get_transformed_point(point, mat4));
        }
        curve
    }

    pub fn get_polyline(&self, query: &DiscreteQuery) -> Vec<f32> {
        let curve = self.get_valid();
        let count = self.get_sample_count(query.count);
        (0..count).into_iter()
            .map(|u| curve.get_vector_at_u(u as f32 / (count-1) as f32)) 
            .flatten().collect()
    }

    pub fn get_controls_as_vec2(&self) -> Vec<Vec2> {
        self.controls.iter().map(|p| {
            vec2(p[0], p[1])
        }).collect()
    }

    pub fn get_sample_count(&self, count: usize) -> usize { 
        let mul = self.controls.len()-1;
        self.controls.len() + count * (self.order - 2) * mul
    }

    pub fn get_vec2_at_u(&self, u: f32) -> Vec2 {
        let p = self.get_vector_at_u(u);
        vec2(p[0], p[1])
    }

    pub fn get_vector_at_u(&self, u: f32) -> Vec<f32> {
        let basis = self.get_rational_basis_at_u(u);
        let mut vector = vec![];
        if !self.controls.is_empty() {
            for component_index in 0..self.controls[0].len() { 
                vector.push(
                    (0..self.controls.len())
                        .map(|i| self.controls[i][component_index] * basis[i]).sum()
                );
            }
        }
        vector
    }

    fn get_rational_basis_at_u(&self, u: f32) -> Vec<f32> {
        let basis = self.get_basis_at_u(u);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    fn get_basis_at_u(&self, normal_u: f32) -> Vec<f32> {
        let u = *self.knots.last().unwrap_or(&0.) * normal_u; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_u(u);
        for degree in 1..self.order {
            for i0 in 0..self.controls.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] != 0. {
                    f = (u - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
                }
                if basis[i1] != 0. {
                    g = (self.knots[degree + i1] - u) / (self.knots[degree + i1] - self.knots[i1])
                }
                basis[i0] = f * basis[i0] + g * basis[i1];
            }
        }
        if normal_u == 1. { 
            basis[self.controls.len() - 1] = 1.; // last control edge case
        }
        basis
    }

    fn get_basis_of_degree_0_at_u(&self, u: f32) -> Vec<f32> {
        self.knots.windows(2)
            .map(|knots| {
                if u >= knots[0] && u < knots[1] {
                    1.
                } else {
                    0.
                }
            }).collect()
    }


    pub fn get_valid(&self) -> CurveShape {
        // log("get valid curve!!!!");
        // console_log!("controls {}", self.controls.len());
        // let order = self.get_valid_order();
        // console_log!("order {}", order);
        // let knots = self.get_valid_knots();
        // console_log!("knots {}", knots.len());
        // let weights = self.get_valid_weights();
        // console_log!("weights {}", weights.len());
        // let controls  = self.controls.clone();
        // console_log!("controls {}", controls.len());
        let curve = CurveShape {
            order: self.get_valid_order(),
            knots: self.get_valid_knots(),
            weights: self.get_valid_weights(),
            controls: self.controls.clone(), //self.controls.iter().map(|c| self.get_valid_control(c)).collect(), // self.controls.clone(), //
            min: self.min,
            max: self.max,
        };
        //console_log!("get_valid curve control count: {}", curve.controls.len());
        curve
    }
    
    // fn get_valid_control(&self, control: &Shape) -> Shape {
    //     match control {
    //         Shape::Point(m) => Shape::Point(*m),
    //         Shape::Curve(m) => Shape::Curve(m.get_valid()),
    //         _ => Shape::Point([0.; 3]),
    //     }
    // }

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
// impl Curve {
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





// let mut order = self.order;
//         if order > self.controls.len() {
//             order = self.controls.len();
//         }
//         if order < 2 {
//             order = 2;
//         }
//         order 



// let mut polylines: Vec<Vec<f32>> = vec![];
//         let mut polyline = vec![];
//         let mut boundaries = vec![];
//         let bound = BoundaryV::default();
//         boundaries.push(&bound);
//         for boundary in &self.boundaries {
//             if let Boundary::V(boundary) = boundary {
//                 boundaries.push(boundary);
//             }
//         }
//         let bound = BoundaryV {v: 1., angle: 0.};
//         boundaries.push(&bound);
//         boundaries.sort_by(|a, b| a.v.partial_cmp(&b.v).unwrap());
//         let v_count = self.get_sample_count(count);
//         let mut stops: Vec<f32> = (0..v_count).map(|step| step as f32 / (v_count-1) as f32).collect();
//         stops.extend(boundaries.iter().map(|b| b.v));
//         stops.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         stops.dedup();
//         // if stops.len() > 3 {
//         //     console_log!("stops count: {}", stops.len());
//         //     console_log!("stops: {}, {}, {}, {}", stops[0], stops[1], stops[2], stops[3]);
//         // }
//         let mut on = false;
//         if boundaries.len() > 2{
//             if boundaries[1].angle > 0. { on = true; }
//         }
//         let mut bi = 0;
//         let mut polyline_in_progress = false;
//         for v in stops {
//             if v >= boundaries[bi].v { 
//                 on = !on;
//                 if on {
//                     polyline = self.get_vector_at_uv(u, boundaries[bi].v);
//                     polyline_in_progress = true;
//                 }else{
//                     if polyline_in_progress {
//                         polyline.extend(self.get_vector_at_uv(u, boundaries[bi].v));
//                         polylines.push(polyline.clone());
//                         polyline_in_progress = false;
//                     }
//                 }
//                 if bi < boundaries.len()-1 { bi += 1; }
//             }
//             if polyline_in_progress {
//                 polyline.extend(self.get_vector_at_uv(u, v));
//             }
//         }
//         polylines