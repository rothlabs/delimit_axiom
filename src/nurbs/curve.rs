
use crate::{Model, Shape, query::DiscreteQuery, get_points, get_transformed_point};
use glam::*;
use serde::{Deserialize, Serialize};

use super::Nurbs;
//use rayon::prelude::*;

// ((a % b) + b) % b)  ->  a modulo b

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

//static default_boundary: BoundaryV = BoundaryV::default();

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Curve::default")]
pub struct Curve {
    pub controls: Vec<Model>,
    pub knots:    Vec<f32>,    // knot_count = order + control_count
    pub weights:  Vec<f32>,    // weight_count = control_count
    pub order:    usize,       // order = polynomial_degree + 1
    //pub min:      f32,
    //pub max:      f32,
}

impl Curve {
    pub fn get_shapes(&self) -> Vec<Shape> {
        vec![Shape::Curve(CurveShape{
            nurbs: Nurbs {
                order:   self.order,
                knots:   self.knots.clone(),
                weights: self.weights.clone(),
            },
            controls: get_points(&self.controls),//.iter().map(|p| vec3(p[0], p[1], p[2])).collect(),
            min: 0., //self.min,
            max: 1., //self.max,
        })]
    }
}

#[derive(Clone)]
pub struct CurveShape {
    pub nurbs:    Nurbs,
    pub controls: Vec<Vec3>, // [f32; 3]
    pub min:      f32,
    pub max:      f32,
}

impl Default for CurveShape {
    fn default() -> Self {
        Self {
            nurbs: Nurbs::default(),
            controls: vec![],
            min: 0.,
            max: 1.,
        }
    }
}

impl CurveShape { // impl<T: Default + IntoIterator<Item=f32>> Curve<T> {
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        let mut curve = Self {
            nurbs: self.nurbs.clone(),
            controls: vec![],
            min: self.min,
            max: self.max,
        };
        for point in &self.controls {
            curve.controls.push(get_transformed_point(point, mat4));
        }
        curve
    }
    
    // pub fn get_param_step(&self, min_count: usize, max_distance: f32) -> f32 {
    //     1. / self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &self.controls) as f32 // self.nurbs.get_param_step(min_count, max_distance, &self.controls)
    // }

    // pub fn get_param_samples(&self, min_count: usize, max_distance: f32) -> Vec<f32> {
    //     self.nurbs.get_param_samples(min_count, max_distance, &self.controls)
    // }

    pub fn get_param_step_and_samples(&self, min_count: usize, max_distance: f32) -> (f32, Vec<f32>) {
        let count = self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &self.controls);
        (1./(count-1) as f32, (0..count).map(|u| u as f32 / (count-1) as f32).collect())
    }

    pub fn get_polyline(&self, query: &DiscreteQuery) -> Vec<f32> {
        let curve = self.get_valid();
        let count = curve.nurbs.get_sample_count(query.count);
        (0..count).into_iter()
            .map(|u| curve.get_vector_at_u(u as f32 / (count-1) as f32)) 
            .flatten().collect()
    }

    pub fn get_vec2_at_u(&self, u: f32) -> Vec2 {
        let p = self.get_vector_at_u(u);
        vec2(p[0], p[1])
    }

    pub fn get_vec3_at_u(&self, u: f32) -> Vec3 {
        let p = self.get_vector_at_u(u);
        vec3(p[0], p[1], p[2])
    }

    pub fn get_vector_at_u(&self, u: f32) -> Vec<f32> {
        let bounded_u = self.min*(1.-u) + self.max*u;
        let basis = self.nurbs.get_rational_basis_at_u(bounded_u);
        let mut vector = vec![];
        if !self.controls.is_empty() {
            for component_index in 0..3 { // self.controls[0].len() { 
                vector.push(
                    (0..self.controls.len())
                        .map(|i| self.controls[i][component_index] * basis[i]).sum()
                );
            }
        }
        vector
    }

    pub fn get_valid(&self) -> CurveShape {
        CurveShape {
            nurbs: self.nurbs.get_valid(self.controls.len()),
            controls: self.controls.clone(), 
            min: self.min,
            max: self.max,
        }
    }
}





// fn get_valid_control(&self, control: &Shape) -> Shape {
    //     match control {
    //         Shape::Point(m) => Shape::Point(*m),
    //         Shape::Curve(m) => Shape::Curve(m.get_valid()),
    //         _ => Shape::Point([0.; 3]),
    //     }
    // }

    // fn get_valid_order(&self) -> usize {
    //     self.order.min(self.controls.len()).max(2)
    // }

    // fn get_valid_weights(&self) -> Vec<f32> {
    //     if self.weights.len() == self.controls.len() {
    //         self.weights.clone()
    //     } else {
    //         vec![1.; self.controls.len()]
    //     }
    // }

    // fn get_valid_knots(&self) -> Vec<f32> {
    //     if self.knots.len() == self.controls.len() + self.get_valid_order() {
    //         self.knots.clone()
    //     } else {
    //         self.get_open_knots()
    //     }
    // }

    // fn get_open_knots(&self) -> Vec<f32> {
    //     let order = self.get_valid_order();
    //     let repeats = order - 1; // knot multiplicity = order for ends of knot vector
    //     let max_knot = self.controls.len() + order - (repeats * 2) - 1;
    //     let mut knots = vec![0_f32; repeats];
    //     knots.extend((0..=max_knot).map(|k| k as f32));
    //     knots.extend(vec![max_knot as f32; repeats]);
    //     knots
    // }




    // pub fn get_param_step(&self, min_count: usize, max_distance: f32) -> f32 {
    //     1. / (self.get_sample_count_with_max_distance(min_count, max_distance) - 1) as f32
    // }

    // pub fn get_param_samples(&self, min_count: usize, max_distance: f32) -> Vec<f32> {
    //     let mut sample_params = vec![];
    //     let count = self.get_sample_count_with_max_distance(min_count, max_distance);
    //     for step in 0..count {
    //         sample_params.push(step as f32 / (count-1) as f32);
    //     }
    //     sample_params
    // }

    // pub fn get_sample_count(&self, count: usize) -> usize { 
    //     let mul = self.controls.len()-1;
    //     self.controls.len() + count * (self.nurbs.order - 2) * mul
    // }

    // pub fn get_sample_count_with_max_distance(&self, min_count: usize, max_distance: f32) -> usize {
    //     let curve = self.get_valid();
    //     let mut distance = 0.;
    //     for step in 0..curve.controls.len()-1 {
    //         let u0 = step as f32 / (curve.controls.len()-1) as f32;
    //         let u1 = (step+1) as f32 / (curve.controls.len()-1) as f32;
    //         let dist = curve.get_vec2_at_u(u0).distance(curve.get_vec2_at_u(u1));
    //         if distance < dist {distance = dist;}
    //     }
    //     let mut count = min_count;
    //     let distance_based_count = (distance / max_distance).ceil() as usize;
    //     if distance_based_count > min_count {count = distance_based_count; }
    //     count = count*(curve.controls.len()-1) + curve.controls.len();
    //     count
    // }





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