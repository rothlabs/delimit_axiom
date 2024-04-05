pub mod curve;
pub mod facet;

use std::f32::consts::PI;

use glam::*;
use serde::{Deserialize, Serialize};

// use crate::log;
// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Nurbs {
    pub sign:     f32,
    pub order:    usize,       // order = polynomial_degree + 1
    pub knots:    Vec<f32>,    // knot_count = order + control_count
    pub weights:  Vec<f32>,    // weight_count = control_count
}

impl Default for Nurbs {
    fn default() -> Self {
        Nurbs {
            sign:    1.,
            order:   2,
            knots:   vec![],
            weights: vec![],    
        }
    }
}

impl Nurbs {
    // pub fn negate(&mut self) -> &mut Self {
    //     self.sign = -self.sign;
    //     self
    // }

    pub fn get_sample_count(&self, count: usize) -> usize { 
        let mul = self.weights.len()-1;
        self.weights.len() + count * (self.order - 2) * mul
    }

    fn get_valid(&self, control_count: usize) -> Self {
        let order = self.order.min(control_count).max(2);
        let mut sign = self.sign;
        if sign.abs() < 1. {sign = 1.;}
        Nurbs {
            sign,
            order,//:   self.get_valid_order(control_count),
            knots:   self.get_valid_knots(control_count, order),
            weights: self.get_valid_weights(control_count),
        }
    }

    fn get_valid_weights(&self, control_count: usize) -> Vec<f32> {
        if self.weights.len() == control_count {
            self.weights.clone()
        } else {
            vec![1.; control_count]
        }
    }

    fn get_valid_knots(&self, control_count: usize, order: usize) -> Vec<f32> {
        if self.knots.len() == control_count + order { 
            let last_knot = self.knots.last().unwrap();
            self.knots.iter().map(|k| k / last_knot).collect()
        } else {
            self.get_open_knots(control_count, order)
        }
    }
    
    pub fn normalize_knots(&mut self) {
        let last_knot = self.knots.last().unwrap();
        self.knots = self.knots.iter().map(|k| k / last_knot).collect();
    }

    fn get_open_knots(&self, control_count: usize, order: usize) -> Vec<f32> {
        let repeats = order - 1; // knot multiplicity = order for ends of knot vector
        let max_knot = control_count + order - (repeats * 2) - 1;
        let mut knots = vec![0_f32; repeats];
        knots.extend((0..=max_knot).map(|k| k as f32));
        knots.extend(vec![max_knot as f32; repeats]);
        let last_knot = knots.last().unwrap();
        knots = knots.iter().map(|k| k / last_knot).collect();
        knots
    }


    fn get_knot_index(&self, u: f32) -> Option<usize> {
        for i in 0..self.knots.len()-1 { 
            if u >= self.knots[i] && u < self.knots[i+1] { 
                return Some(i)
            }
        }
        None
    }

    fn get_basis(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
        let mut basis = self.get_unweighted_basis(knot_index, u);
        // let sum: f32 = (0..self.order).map(|k| {
        //     let i = 4 - self.order + k;
        //     basis.0[i] * self.weights[knot_index + i - 3]
        // }).sum();
        let mut sum = (0., 0.);
        for k in 0..self.order {
            let i = 4 - self.order + k;
            let weight = self.weights[knot_index + i - 3];
            sum.0 += basis.0[i] * weight;
            //sum.0 += basis.0[i];
            //sum.1 += basis.1[i].abs() * weight;
        }
        for k in 0..self.order {
            let i = 4 - self.order + k;
            let weight = self.weights[knot_index + i - 3];
            basis.0[i] *= weight / sum.0; // for getting position
            //basis.0[i] /= sum.0; // for getting position
            //basis.1[i] *= weight / sum.1; // for getting velocity
            //basis.1[k] = (basis.1[i] * PI / 2.).sin();
        }
        basis
    }

    fn get_unweighted_basis(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
        let mut basis = ([0., 0., 0., 1.], [0., 0., 0., 1.]);
        for degree in 1..self.order {
            for k in 0..(degree+1) { 
                let i = 3 - degree + k;
                let k0 = knot_index + k - degree;
                let k1 = k0 + 1; 
                let mut position = 0.;
                let mut velocity = 0.;
                if basis.0[i] != 0. {
                    let div = self.knots[degree + k0] - self.knots[k0];
                    position += basis.0[i] * (u - self.knots[k0]) / div; 
                    velocity += basis.0[i] / div;
                }
                if i < 3 && basis.0[i+1] != 0. {
                    let div = self.knots[degree + k1] - self.knots[k1];
                    //basis.1[i+1] = basis.0[i+1] / 
                    position += basis.0[i+1] * (self.knots[degree + k1] - u) / div; 
                    velocity -= basis.0[i+1] / div;
                } 
                //let ci = knot_index + i - 3;
                basis.0[i] = position;// * self.weights[ci];
                basis.1[i] = velocity;// * self.weights[ci];
            }
        }
        basis
    }

    //  b(u) = (k1-u)/(k1-k0) * (k2-u)/(k2-k0)
    // 'b(u) = - (k1-u)/(k1-k0) / (k2-k0)


    fn get_rational_basis_at_u(&self, u: f32) -> Vec<f32> {
        let basis = self.get_basis_at_u(u);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    fn get_basis_at_u(&self, u: f32) -> Vec<f32> {
        //let u = self.knots.last().unwrap() * normal_u; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_u(u);
        for span in 1..self.order {
            for i0 in 0..self.weights.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] != 0. {
                    f = (u - self.knots[i0]) / (self.knots[span + i0] - self.knots[i0]);
                }
                if basis[i1] != 0. {
                    g = (self.knots[span + i1] - u) / (self.knots[span + i1] - self.knots[i1]); 
                }
                basis[i0] = f * basis[i0] + g * basis[i1];
            }
        }
        if u == 1. { 
            basis[self.weights.len() - 1] = 1.; // last control edge case
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
}





// fn get_basis_at_u(&self, normal_u: f32) -> Vec<f32> {
//     let u = *self.knots.last().unwrap_or(&0.) * normal_u; // .unwrap_throw("") to js client
//     let mut basis = self.get_basis_of_degree_0_at_u(u);
//     for degree in 1..self.order {
//         for i0 in 0..self.weights.len() {
//             let i1 = i0 + 1; 
//             let mut f = 0.;
//             let mut g = 0.;
//             if basis[i0] != 0. {
//                 f = (u - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
//             }
//             if basis[i1] != 0. {
//                 g = (self.knots[degree + i1] - u) / (self.knots[degree + i1] - self.knots[i1])
//             }
//             basis[i0] = f * basis[i0] + g * basis[i1];
//         }
//     }
//     if normal_u == 1. { 
//         basis[self.weights.len() - 1] = 1.; // last control edge case
//     }
//     basis
// }

// fn get_basis_of_degree_0_at_u(&self, u: f32) -> Vec<f32> {
//     self.knots.windows(2)
//         .map(|knots| {
//             if u >= knots[0] && u < knots[1] {
//                 1.
//             } else {
//                 0.
//             }
//         }).collect()
// }






// pub fn get_sample_count_with_max_distance(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> usize {
//     //let curve = self.get_valid(controls.len());
//     let mut distance = 0.;
//     for c in controls.windows(2) {
//         let dist = c[0].distance(c[1]);
//         if distance < dist {distance = dist;}
//     }
//     let distance_based_count = (distance / max_distance).floor() as usize;
//     let mut count = min_count;
//     if distance_based_count > min_count {count = distance_based_count; }
//     count = count*(controls.len()-1) + controls.len();
//     count
// }



    // pub fn get_param_step(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> f32 {
    //     1. / (self.get_sample_count_with_max_distance(min_count, max_distance, controls) - 1) as f32
    // }

    // pub fn get_param_samples(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> Vec<f32> {
    //     let count = self.get_sample_count_with_max_distance(min_count, max_distance, controls);
    //     (0..count).map(|s| s as f32 / (count-1) as f32).collect()
    // }