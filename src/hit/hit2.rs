use std::f32::EPSILON;
use glam::*;
use crate::{log, CurveShape, Spatial3, AT_0_TOL, AT_1_TOL, UV_MISS_BUMP, DOT_1_TOL, DUP_TOL, HIT_TOL};

use super::Miss;

pub struct HitTester2 {
    pub curves: (CurveShape, CurveShape),
    pub spatial:      Spatial3,
    pub points:       Vec<Vec3>,
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

pub enum HitMiss2 {
    Hit(Hit2),
    Miss((Miss, Miss)),
}

impl HitTester2 { 
    pub fn test(&mut self, start_u0: f32, start_u1: f32) -> Option<HitMiss2> { 
        let mut u0 = start_u0;
        let mut u1 = start_u1;
        let mut p0 = self.curves.0.get_point(u0);
        let mut p1 = self.curves.1.get_point(u1);
        for _ in 0..8 {
            if p0.distance(p1) < EPSILON {
                break;
            }
            let target = self.get_tangent_intersection(u0, u1);
            let (u0_t0, p0_t0) = self.curves.0.get_u_and_point_from_target(u0, target - p0);
            let (u1_t0, p1_t0) = self.curves.1.get_u_and_point_from_target(u1, target - p1);
            let (u0_c, p0_c) = self.curves.0.get_u_and_point_from_target(u0, p1 - p0);
            let (u1_c, p1_c) = self.curves.1.get_u_and_point_from_target(u1, p0 - p1);
            let distances = vec![p0_t0.distance(p1_t0), p1.distance(p0_c), p0.distance(p1_c)];
            let mut min_dist = 10000.;
            let mut i = 3;
            for k in 0..3 {
                if min_dist > distances[k] {
                    min_dist = distances[k];
                    i = k;
                }
            }
            if i < 1 {
                p0 = p0_t0;
                p1 = p1_t0;
                u0 = u0_t0;
                u1 = u1_t0;
            } else if i < 2 {
                p0 = p0_c;
                u0 = u0_c;
            } else {
                p1 = p1_c;
                u1 = u1_c;
            }
        }
        if p0.distance(p1) < HIT_TOL  {
            let center = (p0 + p1) / 2.;
            (u0, p0) = self.curves.0.get_u_and_point_from_target(u0, center - p0);
            (u1, p1) = self.curves.1.get_u_and_point_from_target(u1, center - p1);
            let center = (p0 + p1) / 2.;
            let mut duplicate = false;
                for i in self.spatial.get(&center) {
                    if self.points[i].distance(center) < DUP_TOL {
                        duplicate = true;
                        break;
                    }
                }
            if !duplicate {
                if (u0 > AT_1_TOL && u1 < AT_0_TOL) || (u0 < AT_0_TOL && u1 > AT_1_TOL) {
                    return None;
                }
                let delta0 = self.curves.0.get_arrow(u0).delta.normalize();
                let delta1 = self.curves.1.get_arrow(u1).delta.normalize();
                if delta0.dot(delta1).abs() > 0.9999 {
                    return None;
                }
                //let delta0 = self.curves.0.get_arrow(u0).delta;
                //let delta1 = self.curves.1.get_arrow(u1).delta;
                let cross0 = Vec3::Z.cross(delta0).normalize();
                let cross1 = Vec3::Z.cross(delta1).normalize();
                self.spatial.insert(&center, self.points.len());
                self.points.push(center);
                return Some(HitMiss2::Hit(Hit2{
                    center,
                    hit: (CurveHit {u:u0, dot:cross0.dot(delta1)}, 
                          CurveHit {u:u1, dot:cross1.dot(delta0)}),
                }))
            }
            
        } 
        let delta0 = self.curves.0.get_arrow(u0).delta.normalize();
        let delta1 = self.curves.1.get_arrow(u1).delta.normalize();
        let cross0 = Vec3::Z.cross(p1 - p0).normalize();
        let cross1 = Vec3::Z.cross(p0 - p1).normalize();
        if u0 < AT_0_TOL {
            p0 += delta0 * UV_MISS_BUMP;
        } else if u0 > AT_1_TOL {
            p0 -= delta0 * UV_MISS_BUMP;
        }
        if u1 < AT_0_TOL {
            p1 += delta1 * UV_MISS_BUMP;
        } else if u1 > AT_1_TOL {
            p1 -= delta1 * UV_MISS_BUMP;
        }
        let distance = p0.distance(p1);
        Some(HitMiss2::Miss((
            Miss{distance, dot:cross0.dot(delta1)},
            Miss{distance, dot:cross1.dot(delta0)}, 
        )))
    }

    pub fn get_tangent_intersection(&self, u0: f32, u1: f32) -> Vec3 {
        let arrow0 = self.curves.0.get_arrow(u0);
        let arrow1 = self.curves.1.get_arrow(u1);
        if arrow1.delta.length() < 0.00001 {
            console_log!("u1 {}", u1);
            panic!("hi2.get_tengent_intersection arrow1.delta is 0!");
        }
        arrow0.middle(&arrow1)
    }
}