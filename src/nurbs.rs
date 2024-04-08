pub mod curve;
pub mod facet;

use glam::*;
use serde::{Deserialize, Serialize};

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
        let mut basis = ([0., 0., 0., 1.], [0., 0., 0., 1.]);
        let r1 = self.knots[knot_index - 1];
        let k0 = self.knots[knot_index];
        let k1 = self.knots[knot_index + 1];
        let k2 = self.knots[knot_index + 2];
        let k1u = k1 - u;
        let uk0 = u - k0;
        let k0k1 = k0 - k1;
        let k1k0 = k1 - k0;
        if self.order > 2 { // quadratic
            let w0 = self.weights[knot_index - self.order + 1];
            let w1 = self.weights[knot_index - self.order + 2];
            let w2 = self.weights[knot_index - self.order + 3];
            let k0u = k0 - u;
            let k2u = k2 - u;
            let ur1 = u - r1;
            let r1k2 = r1 - k2;
            let k0k2 = k0 - k2;
            let k1r1 = k1 - r1;
            let k2k0 = k2 - k0;
            let p0 = k1u/k1k0 * k1u/k1r1 * w0;
            let p1 = (k1u/k1k0 * ur1/k1r1 + uk0/k1k0 * k2u/k2k0) * w1;
            let p2 = uk0/k1k0 * uk0/k2k0 * w2;
            let sum = p0 + p1 + p2;
            basis.0 = [0., p0/sum, p1/sum, p2/sum];
            let a0 = 2. * k0k1 * k0k2 * k1r1;
            let w0xk1u = w0 * k1u;
            let w2xuk0 = w2 * uk0;
            let n0 = a0 * w0xk1u * (w1 * (u-k2) - w2xuk0);
            let n1 = a0 * w1 * (w0 * k1u * k2u - w2xuk0 * ur1);
            let n2 = a0 * w2xuk0 * (w0 * k1u + w1 * ur1);
            let uxu = u * u;
            let k2xr1 = k2 * r1;
            let ux2 = u * 2.;
            let a1 = - w0xk1u * k0k2 * k1u + w1 * (k0 * (k1 * r1k2 + k2xr1 - r1 * ux2 + uxu) - k1*(k2xr1 - k2 * ux2 + uxu) + uxu * r1k2);
            let d0 = a1 + w2xuk0 * uk0 * k1r1;
            let d1 = a1 + w2 * k0u * k0u * k1r1;
            basis.1 = [0., n0/d0/d0, n1/d0/d0, n2/d1/d1];
        } else { // linear
            basis.0 = [0., 0., k1u/k1k0, uk0/k1k0];
            basis.1 = [0., 0., 1./k0k1, 1./k1k0];
        }
        basis
    }
}


//((k1-u)/(k1-k0)*(k1-u)/(k1-r1)*a) / k1u/k1k0*k1u/k1r1*a + 

// fn get_basis(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
//     let mut basis = ([0., 0., 0., 1.], [0., 0., 0., 1.]);
//     let r1 = self.knots[knot_index - 1];
//     let k0 = self.knots[knot_index];
//     let k1 = self.knots[knot_index + 1];
//     let k2 = self.knots[knot_index + 2];
//     let k1u = k1 - u;
//     let uk0 = u - k0;
//     let k0k1 = k0 - k1;
//     let k1k0 = k1 - k0;
//     if self.order > 2 { // quadratic
//         let w0 = self.weights[knot_index - self.order + 1];
//         let w1 = self.weights[knot_index - self.order + 2];
//         let k0u = k0 - u;
//         let k2u = k2 - u;
//         let ur1 = u - r1;
//         let r1k2 = r1 - k2;
//         let k0k2 = k0 - k2;
//         let k1r1 = k1 - r1;
//         let k2k0 = k2 - k0;
//         let w2 = self.weights[knot_index - self.order + 3];
//         let p0 = k1u/k1k0 * k1u/k1r1 * w0;
//         let p1 = (k1u/k1k0 * ur1/k1r1 + uk0/k1k0 * k2u/k2k0) * w1;
//         let p2 = uk0/k1k0 * uk0/k2k0 * w2;
//         let sum = p0 + p1 + p2;
//         basis.0 = [0., p0/sum, p1/sum, p2/sum];
//         let a0 = 2. * k0k1 * k0k2 * k1r1;
//         //let w0xk1u = w0 * k10;
//         //let w2xuk0 = w2 * uk0;
//         let n0 = w0 * a0 * k1u * (w1 * (u-k2) - w2 * uk0);
//         let n1 = w1 * a0 * (w0 * k1u * k2u - w2 * uk0 * ur1);
//         let n2 = w2 * a0 * uk0 * (w0 * k1u + w1 * ur1);
//         let uxu = u * u;
//         let k2xr1 = k2 * r1;
//         let ux2 = u * 2.;
//         let a1 = - w0 * k0k2 * k1u * k1u + w1 * (k0 * (k1 * r1k2 + k2xr1 - r1 * ux2 + uxu) - k1*(k2xr1 - k2 * ux2 + uxu) + uxu * r1k2);
//         let d0 = a1 + w2 * uk0 * uk0 * k1r1;
//         let d1 = a1 + w2 * k0u * k0u * k1r1;
//         basis.1 = [0., n0/d0/d0, n1/d0/d0, n2/d1/d1];
//     } else { // linear
//         basis.0 = [0., 0., k1u/k1k0, uk0/k1k0];
//         basis.1 = [0., 0., 1./k0k1, 1./k1k0];
//     }
//     basis
// }




// fn get_rational_basis_at_u(&self, u: f32) -> Vec<f32> {
    //     let basis = self.get_basis_at_u(u);
    //     let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
    //     if sum > 0. {
    //         self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
    //     } else {
    //         vec![0.; self.weights.len()]
    //     }
    // }

    // fn get_basis_at_u(&self, u: f32) -> Vec<f32> {
    //     //let u = self.knots.last().unwrap() * normal_u; // .unwrap_throw("") to js client
    //     let mut basis = self.get_basis_of_degree_0_at_u(u);
    //     for span in 1..self.order {
    //         for i0 in 0..self.weights.len() {
    //             let i1 = i0 + 1; 
    //             let mut f = 0.;
    //             let mut g = 0.;
    //             if basis[i0] != 0. {
    //                 f = (u - self.knots[i0]) / (self.knots[span + i0] - self.knots[i0]);
    //             }
    //             if basis[i1] != 0. {
    //                 g = (self.knots[span + i1] - u) / (self.knots[span + i1] - self.knots[i1]); 
    //             }
    //             basis[i0] = f * basis[i0] + g * basis[i1];
    //         }
    //     }
    //     if u == 1. { 
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



// fn get_basis_old(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
//     let mut basis = self.get_unweighted_basis(knot_index, u);
//     // let sum: f32 = (0..self.order).map(|k| {
//     //     let i = 4 - self.order + k;
//     //     basis.0[i] * self.weights[knot_index + i - 3]
//     // }).sum();
//     let mut sum = (0., 0.);
//     for k in 0..self.order {
//         let i = 4 - self.order + k;
//         let weight = self.weights[knot_index + i - 3];
//         sum.0 += basis.0[i] * weight;
//         //sum.1 += weight.powf(2.);
//         //sum.1 += basis.1[i].abs() * weight;
//     }
//     //sum.1 = sum.1.sqrt();
//     //let order_length = (self.order as f32).sqrt();
//     for k in 0..self.order {
//         let i = 4 - self.order + k;
//         let weight = self.weights[knot_index + i - 3];
//         basis.0[i] *= weight / sum.0; // for getting position
//         // if basis.1[i] > 0. {
//         //     basis.1[i] *= weight * (order_length / sum.1); // for getting velocity
//         // }  else {
//         //     basis.1[i] *= weight / (order_length / sum.1); // for getting velocity
//         // }
//         //basis.0[i] /= sum.0; // for getting position
//         //basis.1[k] = (basis.1[i] * PI / 2.).sin();
//     }
//     basis
// }


// fn get_unweighted_basis2(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
//     let mut basis = ([0., 0., 0., 1.], [0., 0., 0., 1.]);
//     let r1 = self.knots[knot_index - 1];
//     let k0 = self.knots[knot_index];
//     let k1 = self.knots[knot_index + 1];
//     let k2 = self.knots[knot_index + 2];
//     let ci = knot_index - self.order + 1;
//     if self.order > 3 { // cubic
//         let r2 = self.knots[knot_index - 2];
//         let k3 = self.knots[knot_index + 3];
//     }else if self.order > 2 { // quadratic
//         basis.0 = [
//             0., 
//             (k1-u)/(k1-k0) * (k1-u)/(k1-r1) * self.weights[ci], 
//             ((k1-u)/(k1-k0) * (u-r1)/(k1-r1) + (u-k0)/(k1-k0) * (k2-u)/(k2-k0)) * self.weights[ci+1], 
//             (u-k0)/(k1-k0) * (u-k0)/(k2-k0) * self.weights[ci+2],
//         ];
//         basis.0 = [
//             0., 
//             basis.0[1] / (basis.0[1]+basis.0[2]+basis.0[3]), 
//             basis.0[2] / (basis.0[1]+basis.0[2]+basis.0[3]), 
//             basis.0[3] / (basis.0[1]+basis.0[2]+basis.0[3]), 
//         ];
//         basis.1 = [
//             0., 
//             (2. * (k1-u) / (k0-k1) / (k1-r1)), 
//             (2. * (k0 * (u-r1) + k1 * (k2-u) + u * (r1-k2)) / (k0-k1) / (k0-k2) / (k1-r1)), 
//             (2. * (u-k0) / (k0-k1) / (k0-k2)),
//         ];
//     } else { // linear
//         basis.0 = [0., 0., (k1-u)/(k1-k0), (u-k0)/(k1-k0)];
//         basis.1 = [0., 0., 1./(k0-k1), 1./(k1-k0)];
//     }
//     basis
// }


// fn get_unweighted_basis(&self, knot_index: usize, u: f32) -> ([f32; 4], [f32; 4]) {
//     let mut basis = ([0., 0., 0., 1.], [0., 0., 0., 1.]);
//     for degree in 1..self.order {
//         for k in 0..degree+1 { 
//             let i = 3 - degree + k;
//             let k0 = knot_index - degree + k; 
//             let mut position = 0.;
//             let mut velocity = 0.;
//             if basis.0[i] != 0. {
//                 let div = self.knots[degree + k0] - self.knots[k0];
//                 position += basis.0[i] / div * (u - self.knots[k0]); 
//                 velocity += basis.0[i] / div;
//             }
//             if i < 3 && basis.0[i+1] != 0. {
//                 let k1 = k0 + 1;
//                 let div = self.knots[degree + k1] - self.knots[k1];
//                 position += basis.0[i+1] / div * (self.knots[degree + k1] - u); 
//                 velocity -= basis.0[i+1] / div;
//             } 
//             // let ci = knot_index - self.order + 1 + k;
//             basis.0[i] = position;// * self.weights[ci];
//             basis.1[i] = velocity * (self.order as f32 - 1.);// * self.weights[ci];
//         }
//     }
//     basis
// }



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