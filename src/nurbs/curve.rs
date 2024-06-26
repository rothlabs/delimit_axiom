
use std::f32::consts::PI;
use std::f32::EPSILON;
use crate::{log, FacetShape};
use crate::{get_points, get_reshaped_point, get_vector_hash, query::DiscreteQuery, arrow::Arrow, scene::Polyline, Model, Shape};
use glam::*;
use serde::{Deserialize, Serialize};
use super::Nurbs;

// ((a % b) + b) % b)  ->  a modulo b

const TWO_CONTROL_POINTS: &str = "There should be two control points or more.";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] 
pub struct Curve {
    pub controls: Vec<Model>,
    pub nurbs: Nurbs,
    pub min:  f32,
    pub max:  f32,
    pub arrows: usize,
}

impl Default for Curve {
    fn default() -> Self {
        Self {
            controls: vec![],  
            nurbs: Nurbs::default(),
            min: 0.,
            max: 1.,  
            arrows: 0,
        }
    }
}

impl Curve {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![Shape::Curve(CurveShape{
            controls: get_points(&self.controls),
            nurbs: self.nurbs.clone(),
            min: self.min, 
            max: self.max, 
            rectifier: None,
        }.get_valid())];

        //let mut shapes = revolve.get_shapes();
        if self.arrows > 0 {
            //let count = 30;
            if let Shape::Curve(circle) = shapes[0].clone() {
                for i in 0..self.arrows {
                    let mut curve = CurveShape::default();
                    let ray = circle.get_arrow(i as f32 / (self.arrows - 1) as f32);
                    curve.controls.push(ray.point);
                    curve.controls.push(ray.point + ray.delta);
                    shapes.push(Shape::Curve(curve.get_valid()));
                }
            }
        }
        shapes
    }
}

#[derive(Clone)]
pub struct CurveRectifier {
    pub curve: CurveShape,
    pub facet: FacetShape,
}

#[derive(Clone)]
pub struct CurveShape {
    pub controls: Vec<Vec3>,
    pub nurbs: Nurbs,
    pub min:  f32,
    pub max:  f32,
    pub rectifier: Option<Box<CurveRectifier>>,
}

impl Default for CurveShape {
    fn default() -> Self {
        Self {
            controls: vec![],  
            nurbs: Nurbs::default(),
            min: 0.,
            max: 1.,  
            rectifier: None,
        }
    }
}

impl CurveShape {
    pub fn from_order(order: usize) -> Self {
        let mut curve = Self::default();
        curve.nurbs.order = order;
        curve.nurbs.knots.extend(vec![0.; order]);
        curve
    }

    pub fn from_nurbs_and_controls(nurbs: Nurbs, controls: Vec<Vec3>) -> Self {
        let mut curve = Self::default();
        curve.nurbs = nurbs;
        curve.controls = controls;
        curve
    }

    pub fn negate(&mut self) -> &mut Self {
        self.nurbs.sign = -self.nurbs.sign;
        self
    }

    pub fn reverse(&mut self) -> &mut Self{
        //let max_knot = *self.nurbs.knots.last().unwrap(); 
        self.nurbs.knots.reverse();
        for i in 0..self.nurbs.knots.len() {
            self.nurbs.knots[i] = 1. - self.nurbs.knots[i];
        }
        self.nurbs.weights.reverse();
        self.controls.reverse();
        let min_basis = self.min;
        self.min = 1.-self.max;
        self.max = 1.-min_basis;
        self
    }

    pub fn get_reverse(&self) -> Self {
        let mut curve = self.clone();
        curve.reverse();
        curve
    }

    pub fn reshape(&mut self, mat4: Mat4) -> &mut Self {
        let controls = self.controls.clone();
        self.controls.clear();
        for point in controls {
            self.controls.push(get_reshaped_point(&point, mat4));
        }
        self
    }

    pub fn get_reshape(&self, mat4: Mat4) -> Self {
        let mut curve = self.clone();
        curve.reshape(mat4);
        curve
    }

    pub fn get_unique_knots(&self) -> Vec<f32> {
        let mut knots = vec![0.];
        for k in self.nurbs.knots.windows(2) {
            if k[0] < k[1] {
                knots.push(k[1]);
            }
        }
        knots
    }

    pub fn get_u_and_point_from_target(&self, u: f32, target: Vec3) -> (f32, Vec3) {
        let ray = self.get_arrow(u);
        let length_ratio = target.length() / ray.delta.length();
        let u_dir = ray.delta.normalize().dot(target.normalize()) * length_ratio;

        let mut u1 = u;
        // if u_dir.is_nan() {
        //     console_log!("u_dir is nan!! {}", target.length());
        // }
        if u_dir.abs() > EPSILON {// step.abs() {
            u1 = u + u_dir; 
        }
        u1 = u1.clamp(0., 1.); 
        let point = self.get_point(u1);
        (u1, point)
    }

    pub fn get_polyline(&self, query: &DiscreteQuery) -> Polyline {
        let vector = self.get_polyline_vector(query);
        Polyline {
            digest: get_vector_hash(&vector),
            vector,
        }
    }

    pub fn get_polyline_vector(&self, query: &DiscreteQuery) -> Vec<f32> {
        let curve = self.get_valid();
        let count = curve.nurbs.get_sample_count(query.count);
        (0..count).into_iter()
            .map(|u| curve.get_point(u as f32 / (count-1) as f32).to_array()) 
            .flatten().collect()
    }

    pub fn get_point(&self, u: f32) -> Vec3 {
        let u = self.min * (1.-u) + self.max * u;
        let ki = self.nurbs.get_knot_index(u);      
        let basis = self.nurbs.get_basis(ki, u);
        return (0..self.nurbs.order).map(|k| {
            let i = 4 - self.nurbs.order + k;
            self.controls[ki + i - 3] * basis.0[i]
        }).sum()
    }

    pub fn get_arrow(&self, u: f32) -> Arrow {
        let mut ray = Arrow::new(Vec3::ZERO, Vec3::ZERO);
        let u = self.min * (1.-u) + self.max * u;    
        let ki = self.nurbs.get_knot_index(u);  
        let basis = self.nurbs.get_basis(ki, u);
        for k in 0..self.nurbs.order {
            let i = 4 - self.nurbs.order + k;
            let ci = ki - 3 + i;
            ray.point += self.controls[ci] * basis.0[i];
            ray.delta += self.controls[ci] * basis.1[i];
        }
        ray.delta = ray.delta * (self.max - self.min);
        ray
    }

    pub fn set_min(&mut self, u: f32) {
        self.min = self.min*(1.-u) + self.max*u;
    }

    pub fn set_max(&mut self, min_basis: f32, u: f32) {
        self.max = min_basis*(1.-u) + self.max*u;
    }

    pub fn get_valid(&self) -> CurveShape {
        CurveShape {
            nurbs: self.nurbs.get_valid(self.controls.len()),
            controls: self.controls.clone(), 
            min: self.min,
            max: self.max,
            rectifier: self.rectifier.clone(),
        }
    }
}



    // pub fn get_tangent_at_u(&self, u: f32) -> Vec3 {
    //     let mut step = 0.0001; // 0.0001
    //     if u + step > 1. {step = -step;}
    //     let p0 = self.get_point(u);
    //     let p1 = self.get_point(u + step);
    //     // if let Some(_) = (p1 - p0).try_normalize() {

    //     // }else{
    //     //     log("failed to normalize");
    //     // }
    //     (p1 - p0).normalize() * step.signum()
    // }

    // pub fn get_u_and_point_from_target(&self, u: f32, target: Vec3) -> (f32, Vec3) {
        // let mut step = 0.0001;
        // if u + step > 1. {step = -step;}
        // let p0 = self.get_point(u);
        // let p1 = self.get_point(u + step);
        // let length_ratio = (target.length() / p0.distance(p1)) * step;
        // let u_dir = (p1-p0).normalize().dot(target.normalize()) * length_ratio;


        // pub fn get_inflection_params(&self) -> Vec<f32> {
        //     if self.nurbs.order == 2 && self.controls.len() == 2 {
        //         return vec![0., 0.5, 1.];
        //     }
        //     let mut knots = vec![0.];
        //     //let last_knot = self.nurbs.knots.last().unwrap();
        //     if self.controls.len() > 1 {
        //         let mut direction_basis = (self.controls[1].truncate() - self.controls[0].truncate()).normalize();
        //         let mut turn_basis = 0.;
        //         for i in 1..self.controls.len()-1 {
        //             let dir = (self.controls[i+1].truncate() - self.controls[i].truncate()).normalize();
        //             let turn = direction_basis.angle_between(dir);
        //             if (turn_basis < -0.01 && turn > 0.01) || (turn_basis > 0.01 && turn < -0.01) {
        //                 let u0 = self.nurbs.knots[self.nurbs.order + i - 2];// / last_knot;
        //                 let u1 = self.nurbs.knots[self.nurbs.order + i - 1];// / last_knot;
        //                 let u = (u0 + u1) / 2.;
        //                 if u >= self.min && u <= self.max {
        //                     knots.push(u);
        //                 }
        //             }
        //             direction_basis = dir;
        //             turn_basis = turn;
        //         }
        //     }
        //     //knots.push(0.3);
        //     knots.push(0.5);
        //     //knots.push(0.8);
        //     knots.push(1.);
        //     knots
        // }


                    // if ci > 0 && i > 1 {
                    //     // let ki0 = ci + self.nurbs.order - 2;
                    //     // if self.nurbs.knots[ki0-1] == self.nurbs.knots[ki0] || self.nurbs.knots[ki0+1] == self.nurbs.knots[ki0] {

                    //     // }
                    //     // if ci == 1 {
                    //     //     ray.vector[component] += 
                    //     //         (self.controls[ci][component] - self.controls[ci-1][component]) * basis.0[i] * 2.;
                    //     // }else{
                    //         ray.vector[component] += 
                    //             (self.controls[ci][component] - self.controls[ci-1][component]) * basis.0[i];
                    //     //}
                    // }


                    // //let mut div = 0.;
                    // if k > 0 {
                    //     //div += 1.;
                    //     ray.vector[component] += 
                    //         (self.controls[ci][component] - self.controls[ci-1][component]) * basis[i];
                    // }
                    // if k < self.nurbs.order-1 {
                    //     //div += 1.;
                    //     ray.vector[component] += 
                    //         (self.controls[ci+1][component] - self.controls[ci][component]) * basis[i];
                    // }
                    // //ray.vector[component] *= basis[i] / 2.;



// pub fn get_point(&self, u: f32) -> Vec3 {
//     let p = self.get_vector_at_u(u);
//     vec3(p[0], p[1], p[2])
// }


// pub fn get_vector_at_u(&self, u: f32) -> Vec<f32> {
    //     let bounded_u = self.min*(1.-u) + self.max*u;
    //     let basis = self.nurbs.get_rational_basis_at_u(bounded_u);
    //     let mut vector = vec![];
    //     if !self.controls.is_empty() {
    //         for component_index in 0..3 { // self.controls[0].len() { 
    //             vector.push(
    //                 (0..self.controls.len())
    //                     .map(|i| self.controls[i][component_index] * basis[i]).sum()
    //             );
    //         }
    //     }
    //     vector
    // }


// pub fn remove_doubles(&mut self, tolerance: f32) {
    //     let mut controls = vec![*self.controls.first().unwrap()];
    //     let last_point = *self.controls.last().unwrap();
    //     let mut prev_point = controls[0];
    //     for i in 1..self.controls.len()-1 {
    //         if self.controls[i].distance(prev_point) > tolerance && self.controls[i].distance(last_point) > tolerance {
    //             controls.push(self.controls[i]);
    //             prev_point = self.controls[i];
    //         }
    //     }
    //     controls.push(last_point);
    //     self.controls = controls;
    // }

    // pub fn set_knots_by_control_distance(&mut self) {
    //     self.nurbs.knots = vec![0.; self.nurbs.order];
    //     let mut distance = 0.;
    //     for i in 1..self.controls.len() {
    //         distance += self.controls[i-1].distance(self.controls[i]);
    //         self.nurbs.knots.push(distance);
    //     }
    //     self.nurbs.knots.extend(vec![distance as f32; self.nurbs.order-1]);
    // }



// let mut last_active_knot_i = 0;
//             let mut basis = vec![0.; 4];
//             let mut basis_i = 0;
//             let mut basis_shift = 0;
//             let mut basis_started = false;
//             for i in 0..self.nurbs.knots.len()-1 { 
//                 if u >= self.nurbs.knots[i] && u < self.nurbs.knots[i+1] { 
//                     basis[basis_i] = 1.;
//                     basis_started = true;
//                     last_active_knot_i = i;
//                     basis_shift = 3-basis_i;
//                 }else{
//                     basis[basis_i] = 0.;
//                 }
//                 if basis_started {basis_i += 1;}
//                 if basis_i > 3 {break;}
//             }
//             for i in 0..(4-basis_shift) {
//                 basis[3-i] = basis[3-i-basis_shift];
//                 basis[3-i-basis_shift] = 0.;
//             }
//             if last_active_knot_i < 1 {
//                 last_active_knot_i = self.nurbs.knots.len() - self.nurbs.order - 1;
//                 basis = vec![0., 0., 0., 1.];
//             }
//             for span in 1..self.nurbs.order {
//                 for k in 0..self.nurbs.order { 
//                     let i = (4-self.nurbs.order) + k;
//                     let i0 = last_active_knot_i - self.nurbs.order + k + 1;
//                     let i1 = i0 + 1;  
//                     if basis[i] != 0. {
//                         basis[i] += basis[i] * ((u - self.nurbs.knots[i0]) / (self.nurbs.knots[span + i0] - self.nurbs.knots[i0]));
//                     }
//                     if i < 3 && basis[i+1] != 0. {
//                         basis[i] += basis[i+1] * ((self.nurbs.knots[span + i1] - u) / (self.nurbs.knots[span + i1] - self.nurbs.knots[i1]));
//                     }
//                 }
//             }
//             let sum: f32 = (0..self.nurbs.order).map(|k| {
//                 let i = (4-self.nurbs.order) + k;
//                 let ci = last_active_knot_i - self.nurbs.order + k + 1;
//                 basis[i] * self.nurbs.weights[ci]
//             }).sum();
//             for comp_i in 0..3 { 
//                 //console_log!("order {}", self.nurbs.order);
//                 //console_log!("last_active_knot_i {}", last_active_knot_i);
//                 vector.push(
//                     (0..self.nurbs.order).map(|k| {
//                         let i = (4-self.nurbs.order) + k;
//                         let ci = last_active_knot_i - self.nurbs.order + k + 1;
//                         self.controls[ci][comp_i] * self.nurbs.weights[ci] * basis[i] / sum
//                     }).sum()
//                 );
//             }








// let knot_len = self.nurbs.knots.len()-1;
//             for i in 0..knot_len { // (self.nurbs.order-1)
//                 if u >= self.nurbs.knots[knot_len-i-1] && u < self.nurbs.knots[knot_len-i] { 
//                     if basis_count < 1 {last_active = knot_len-i-1;}
//                     basis[3-basis_count] = 1.;
//                     basis_count += 1;
//                     if basis_count > 3 {break;}
//                 }
//             }


// pub fn get_vector_at_u(&self, u: f32) -> Vec<f32> {
//     let bounded_u = self.min*(1.-u) + self.max*u;
//     let basis = self.nurbs.get_rational_basis_at_u(bounded_u);
//     let mut vector = vec![];
//     if !self.controls.is_empty() {
//         for component_index in 0..3 { // self.controls[0].len() { 
//             vector.push(
//                 (0..self.controls.len())
//                     .map(|i| self.controls[i][component_index] * basis[i]).sum()
//             );
//         }
//     }
//     vector
// }

// pub fn get_normalized_knots(&self) -> Vec<f32> {
//     let mut knots = vec![0.];
//     let last_knot = self.nurbs.knots.last().unwrap();
//     for i in 0..self.controls.len()-2 {
//         let u = self.nurbs.knots[self.nurbs.order + i] / last_knot;
//         if u > self.min && u < self.max {
//             knots.push(u);
//         }
//     }
//     knots.push(1.);
//     knots
// }



// pub fn get_param_step(&self, min_count: usize, max_distance: f32) -> f32 {
    //     1. / self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &self.controls) as f32 // self.nurbs.get_param_step(min_count, max_distance, &self.controls)
    // }
    // pub fn get_param_samples(&self, min_count: usize, max_distance: f32) -> Vec<f32> {
    //     self.nurbs.get_param_samples(min_count, max_distance, &self.controls)
    // }
    // pub fn get_param_step_and_samples(&self, min_count: usize, max_distance: f32) -> (f32, Vec<f32>) {
    //     let count = self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &self.controls);
    //     (1./(count-1) as f32, (0..count).map(|u| u as f32 / (count-1) as f32).collect())
    // }
        // pub fn get_vec2_at_u(&self, u: f32) -> Vec2 {
    //     let p = self.get_vector_at_u(u);
    //     vec2(p[0], p[1])
    // }



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