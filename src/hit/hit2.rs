use std::f32::{EPSILON, INFINITY};

use crate::{log, Ray, CurveShape, Spatial3};
use glam::*;

use super::Miss;

//#[derive(Clone)]
pub struct HitTester2 {
    pub curves: (CurveShape, CurveShape),
    pub spatial:      Spatial3,
    pub points:       Vec<Vec3>,
    pub tolerance:    f32,
    pub duplication_tolerance: f32,
}

#[derive(Clone)]
pub struct Hit2 {
    pub hit: (CurveHit, CurveHit),
    pub center: Vec3,
}

#[derive(Clone)]
pub struct CurveHit {
    pub u: f32,
    pub dot: f32,
}

impl HitTester2 { 
    pub fn test(&mut self, start_u0: f32, start_u1: f32) -> Result<Hit2, (Miss, Miss)> { 
        let mut u0 = start_u0;
        let mut u1 = start_u1;
        let mut p0 = self.curves.0.get_point_at_u(u0);
        let mut p1 = self.curves.1.get_point_at_u(u1);
        //let mut center = Vec3::ZERO;
        //let mut distance = INFINITY;
        let mut distance_basis = INFINITY;
        // let mut u0_prev = u0;
        // let mut u1_prev = u1;
        for _ in 0..20 {
            let target = self.get_tangent_intersection(u0, u1, p0, p1);
            let (u0_t0, p0_t0) = self.curves.0.get_u_and_point_from_target(u0, target - p0);
            let (u1_t0, p1_t0) = self.curves.1.get_u_and_point_from_target(u1, target - p1);
            let center = (p0 + p1) / 2.;
            let (u0_t1, p0_t1) = self.curves.0.get_u_and_point_from_target(u0, center - p0);
            let (u1_t1, p1_t1) = self.curves.1.get_u_and_point_from_target(u1, center - p1);

            // let (u0_c, p0_c) = self.curves.0.get_u_and_point_from_target(u0, p1 - p0);
            // let (u1_c, p1_c) = self.curves.1.get_u_and_point_from_target(u1, p0 - p1);

            // let distances = vec![p0_t0.distance(p1_t0), p0_t1.distance(p1_t1), p1.distance(p0_c), p0.distance(p1_c)];
            // let mut min_dist = 10000.;
            // let mut i = 3;
            // for k in 0..4 {
            //     if min_dist > distances[k] {
            //         min_dist = distances[k];
            //         i = k;
            //     }
            // }

            // if i < 1 {
            //     p0 = p0_t0;
            //     p1 = p1_t0;
            //     u0 = u0_t0;
            //     u1 = u1_t0;
            // } else if i < 2 {
            //     p0 = p0_t1;
            //     p1 = p1_t1;
            //     u0 = u0_t1;
            //     u1 = u1_t1;
            // } else if i < 3 {
            //     p0 = p0_c;
            //     u0 = u0_c;
            // } else {
            //     p1 = p1_c;
            //     u1 = u1_c;
            // }

            if p0_t0.distance(p1_t0) < p0_t1.distance(p1_t1) {
                p0 = p0_t0;
                p1 = p1_t0;
                u0 = u0_t0;
                u1 = u1_t0;
            } else {
                p0 = p0_t1;
                p1 = p1_t1;
                u0 = u0_t1;
                u1 = u1_t1;
            }
            let distance = p0.distance(p1);
            if distance < self.tolerance  {
                let center = (p0 + p1) / 2.;
                (u0, p0) = self.curves.0.get_u_and_point_from_target(u0, center - p0);
                (u1, p1) = self.curves.1.get_u_and_point_from_target(u1, center - p1);
                let center = (p0 + p1) / 2.;
                let mut duplicate = false;
                    for i in self.spatial.get(&center) {
                        if self.points[i].distance(center) < self.duplication_tolerance {
                            duplicate = true;
                            //log("duplicate 2D");
                            break;
                        }
                    }
                if !duplicate {
                    
                    let tangent0 = -self.curves.0.get_tangent_at_u(u0);
                    let tangent1 = -self.curves.1.get_tangent_at_u(u1);
                    if tangent0.is_nan() {
                        log("hit tangent0 NaN!!!");
                        //break;
                    }
                    if tangent1.is_nan() {
                        log("hit tangent1 NaN!!!");
                        //break;
                    }
                    if tangent0.dot(tangent1).abs() > 0.995 {
                        return Err((
                            Miss{dot:self.curves.0.nurbs.sign, distance:0.}, // , point: p0 
                            Miss{dot:self.curves.1.nurbs.sign, distance:0.}, // , point: p1
                        ))
                    }
                    let cross0 = Vec3::Z.cross(tangent0).normalize() * self.curves.0.nurbs.sign;
                    let cross1 = Vec3::Z.cross(tangent1).normalize() * self.curves.1.nurbs.sign;
                    self.spatial.insert(&center, self.points.len());
                    self.points.push(center);
                    return Ok(Hit2{
                        hit: (CurveHit {u:u0, dot:cross0.dot(tangent1)}, 
                              CurveHit {u:u1, dot:cross1.dot(tangent0)}),
                        center,
                    })
                }
                break;
            } 
            if distance >= distance_basis {

                // //console_log!("break early! {}", i);
                break;
                // u0 = (u0 + u0_prev) / 2.;
                // u1 = (u1 + u1_prev) / 2.;
                // p0 = self.curves.0.get_point_at_u(u0);
                // p1 = self.curves.1.get_point_at_u(u1);
                // //(u0, p0) = self.curves.0.get_u_and_point_from_target(u0, center - p0);
                // //(u1, p1) = self.curves.1.get_u_and_point_from_target(u1, center - p1);
            }
            // u0 = (u0 + u0_prev) / 2.;
            // u1 = (u1 + u1_prev) / 2.;
            // p0 = self.curves.0.get_point_at_u(u0);
            // p1 = self.curves.1.get_point_at_u(u1);
            distance_basis = distance;
            // u0_prev = u0;
            // u1_prev = u1;
        }
        let tangent0 = self.curves.0.get_tangent_at_u(u0);
        let tangent1 = self.curves.1.get_tangent_at_u(u1);
        let cross0 = Vec3::Z.cross((p1 - p0).normalize()).normalize() * self.curves.0.nurbs.sign;
        let cross1 = Vec3::Z.cross((p0 - p1).normalize()).normalize() * self.curves.1.nurbs.sign;
        if tangent0.is_nan() {
            log("miss tangent0 NaN!");
        }
        if tangent1.is_nan() {
            log("miss tangent1 NaN!");
        }

        // if tangent0.length() < EPSILON {
        //     log("tangent0 is 0!");
        // }
        // if tangent1.length() < EPSILON {
        //     log("tangent1 is 0!");
        // }

        if u0 < EPSILON {
            p0 += tangent0 * self.tolerance * 2.;
        } else if u0 > 1.-EPSILON {
            p0 -= tangent0 * self.tolerance * 2.;
        }
        if u1 < EPSILON {
            p1 += tangent1 * self.tolerance * 2.;
        } else if u1 > 1.-EPSILON {
            p1 -= tangent1 * self.tolerance * 2.;
        }
        let distance = p0.distance(p1);


        Err((
            Miss{dot:cross0.dot(-tangent1), distance}, // , point: p0 
            Miss{dot:cross1.dot(-tangent0), distance}, // , point: p1
        ))
    }

    pub fn get_tangent_intersection(&self, u0: f32, u1: f32, p0: Vec3, p1: Vec3) -> Vec3 {
        let ray0 = Ray::new(p0, self.curves.0.get_tangent_at_u(u0));
        let ray1 = Ray::new(p1, self.curves.1.get_tangent_at_u(u1));
        ray0.middle(&ray1)
    }
}


// if p0.distance(p1) < self.tolerance {
//     let delta = 0.0001;
//     let d0 = u0 + delta;
//     let pd0 = curve0.get_vec2_at_u(d0);
//     let pd1 = curve1.get_vec2_at_u(u1 + delta);
//     if let Some(ip) = get_line_intersection(p0, pd0, p1, pd1) {
//         let ratio = p0.distance(ip) / p0.distance(pd0);
//         let mut u = u0 + (d0-u0)*ratio;
//         let mut point = curve0.get_vec2_at_u(u);
//         let alt_u = u0 + (u0-d0)*ratio;
//         let alt_point = curve0.get_vec2_at_u(alt_u);
//         if alt_point.distance(ip) < point.distance(ip) {
//             u = alt_u;
//             point = alt_point;
//         }
//         let angle = (pd0-p0).angle_between(pd1-p1);
//         Some(Hit2 {
//             u,
//             point,
//             angle,
//         })
//     }else{
//         None
//     }
// }else{
//     None
// }


        // let mut dir0 = curve0.get_param_step(4, self.cell_size/10.);
        // let mut dir1 = curve1.get_param_step(4, self.cell_size/10.);