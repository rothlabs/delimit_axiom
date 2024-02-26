use std::f32::INFINITY;

use crate::{log, get_line_intersection3, CurveShape, Spatial2, Spatial3};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


//#[derive(Clone)]
pub struct HitTester2 {
    pub curves:    Vec<CurveShape>,
    pub index0:    usize,
    pub index1:    usize,
    pub spatial:   Spatial3, // TODO: switch to linear 1D spatial map
    pub points:    Vec<Vec3>,
    pub normal:    Vec3,
    pub tolerance: f32,
    pub duplication_tolerance: f32,
}

//#[derive(Clone)]
pub struct Hit2 {
    pub hit0: CurveHit,
    pub hit1: CurveHit,
    pub center: Vec3,
}

pub struct CurveHit {
    pub u: f32,
    pub dot: f32,
}

pub struct Miss2 {
    pub miss0: CurveMiss,
    pub miss1: CurveMiss,
}

pub struct CurveMiss {
    pub distance: f32,
    pub dot: f32,
}

impl HitTester2 { 
    pub fn test(&mut self, start_u0: f32, start_u1: f32) -> Result<Hit2, Miss2> { 
        let curve0 = &self.curves[self.index0];
        let curve1 = &self.curves[self.index1];
        //let mut did_hit = false;
        let mut u0 = start_u0;
        let mut u1 = start_u1;
        let mut p0 = curve0.get_point_at_u(u0);
        let mut p1 = curve1.get_point_at_u(u1);
        let mut center = Vec3::ZERO;
        let mut distance = INFINITY;
        let mut last_dist = INFINITY;
        for _ in 0..20 {
            let tangent_hit = self.get_tangent_hit(u0, u1, p0, p1);
            let (u0_t0, p0_t0) = curve0.get_u_and_point_from_target(u0, tangent_hit - p0);
            let (u1_t0, p1_t0) = curve1.get_u_and_point_from_target(u1, tangent_hit - p1);
            center = (p0 + p1) / 2.;
            let (u0_t1, p0_t1) = curve0.get_u_and_point_from_target(u0, center - p0);
            let (u1_t1, p1_t1) = curve1.get_u_and_point_from_target(u1, center - p1);
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
            distance = p0.distance(p1);
            if distance < self.tolerance  {
                center = (p0 + p1) / 2.;
                (u0, p0) = curve0.get_u_and_point_from_target(u0, center - p0);
                (u1, p1) = curve1.get_u_and_point_from_target(u1, center - p1);
                center = (p0 + p1) / 2.;
                let mut duplicate = false;
                    for i in self.spatial.get(&center) {
                        if self.points[i].distance(center) < self.duplication_tolerance {
                            duplicate = true;
                            //log("duplicate 2D");
                            break;
                        }
                    }
                if !duplicate {
                    self.spatial.insert(&center, self.points.len());
                    //self.spatial[self.index1].insert(&p1, self.points[self.index0].len());
                    self.points.push(center);
                    //did_hit = true;
                    //self.points[self.index1].push(p1);
                    let tangent0 = curve0.get_tangent_at_u(u0);
                    let tangent1 = curve1.get_tangent_at_u(u1);
                    let cross0 = self.normal.cross(tangent0).normalize();
                    let cross1 = self.normal.cross(tangent1).normalize();
                    return Ok(Hit2{
                        hit0: CurveHit {u:u0, dot:cross0.dot(tangent1)},
                        hit1: CurveHit {u:u1, dot:cross1.dot(tangent0)},
                        center,
                    })
                }
                break;
            } 
            if distance >= last_dist {
                //console_log!("break early! {}", i);
                break;
            }
            last_dist = distance;
        }
        let tangent0 = curve0.get_tangent_at_u(u0);
        let tangent1 = curve1.get_tangent_at_u(u1);
        // let dir0 = (p1 - p0).normalize();
        // let dir1 = (p0 - p1).normalize();
        let cross0 = self.normal.cross(p1 - p0).normalize();
        let cross1 = self.normal.cross(p0 - p1).normalize();
        Err(Miss2{
            miss0: CurveMiss {dot:cross0.dot(tangent1), distance},
            miss1: CurveMiss {dot:cross1.dot(tangent0), distance},
        })
    }

    pub fn get_tangent_hit(&self, u0: f32, u1: f32, p0: Vec3, p1: Vec3) -> Vec3 {
        let curve0 = &self.curves[self.index0];
        let curve1 = &self.curves[self.index1];
        let tangent0 = curve0.get_tangent_at_u(u0);
        let tangent1 = curve1.get_tangent_at_u(u1);
        get_line_intersection3(p0, tangent0, p1, tangent1) // get_line_intersection(p0, tangent0, p1, tangent1)
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