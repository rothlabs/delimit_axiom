use std::f32::consts::PI;

use crate::log;


use super::union3::UnionBasis3;
use rand::Rng;
use glam::*;

#[derive(Clone)]
pub struct Curve_Hit {
    pub u: f32,
    pub point: Vec3,
    pub dot: f32,
}

#[derive(Clone)]
pub struct Facet_Hit {
    pub uv0: Vec2,
    pub point0: Vec3,
    pub uv1: Vec2,
    pub point1: Vec3,
    pub dot: f32,
}

impl UnionBasis3 { 
    pub fn get_facet_hit(&mut self, f0: &usize, f1: &usize, start0: Vec2, start1: Vec2) -> Option<Facet_Hit> {
        let facet0 = &self.facets[*f0];
        let facet1 = &self.facets[*f1];
        let mut move_uv0 = true; 
        let mut uv0 = start0;
        let mut uv1 = start1;
        let mut p0 = facet0.get_point_at_uv(uv0);
        let mut p1 = facet1.get_point_at_uv(uv1);
        let mut dir0 = self.facet_params[f0].step / 10.;
        let mut dir1 = self.facet_params[f1].step / 10.;
        dir0 = dir0.rotate(Vec2::from_angle(self.rng.gen::<f32>()*PI*2.));
        dir1 = dir1.rotate(Vec2::from_angle(self.rng.gen::<f32>()*PI*2.));
        let mut distance = p0.distance(p1);
        for i in 0..self.max_walk_iterations {
            if distance < self.tolerance { 
                break; 
            }
            if i == self.max_walk_iterations-1 {
                log("Hit3 max iterations!");
            }
            if move_uv0 {
                uv0 = (uv0 + dir0).clamp(Vec2::ZERO, Vec2::ONE);
                p0 = facet0.get_point_at_uv(uv0);
            }else{
                uv1 = (uv1 + dir1).clamp(Vec2::ZERO, Vec2::ONE);
                p1 = facet1.get_point_at_uv(uv1);
            }
            let dist = p0.distance(p1);
            if dist >= distance {
                if move_uv0 {
                    dir0 = dir0.perp() * 0.9;
                }else{
                    dir1 = dir1.perp() * 0.9;
                }
                move_uv0 = !move_uv0;
            }
            distance = dist;
        }
        if distance < self.tolerance {
            let delta = 0.0001;
            let normal0 = facet0.get_normal_at_uv(uv0);
            let normal1 = facet1.get_normal_at_uv(uv1);
            let normal_cross = normal0.cross(normal1);
            let cross0 = normal0.cross(normal_cross);
            let cross1 = normal1.cross(normal_cross);
            let center = approx_line_intersection(p0, cross0, p1, cross1);
            
            Some(Facet_Hit {
                uv0,
                point0: center,
                uv1,
                point1: center,
                dot: 0.,
            })
            // let delta = 0.0001;
            // let d0 = u0 + delta;
            // let pd0 = facet0.get_vec3_at_u(d0);
            // let pd1 = facet1.get_vec3_at_u(u1 + delta);
            // if let Some(ip) = get_line_intersection(p0, pd0, p1, pd1) {
            //     let ratio = p0.distance(ip) / p0.distance(pd0);
            //     let mut u = u0 + (d0-u0)*ratio;
            //     let mut point = facet0.get_vec3_at_u(u);
            //     let alt_u = u0 + (u0-d0)*ratio;
            //     let alt_point = facet0.get_vec3_at_u(alt_u);
            //     if alt_point.distance(ip) < point.distance(ip) {
            //         u = alt_u;
            //         point = alt_point;
            //     }
            //     let dot = (pd0-p0).normalize().dot((pd1-p1).normalize());
            //     Some(Facet_Hit {
            //         u,
            //         point,
            //         dot,
            //     })
            // }else{
            //     None
            // }
        }else{
            None
        }
    }
}

fn approx_line_intersection(
    p1: Vec3, d1: Vec3, 
    p2: Vec3, d2: Vec3,
) -> Vec3 {
    let v = p1 - p2;
    let a = d1.dot(d1);
    let b = d1.dot(d2);
    let c = d2.dot(d2);
    let d = d1.dot(v);
    let e = d2.dot(v);

    let denom = a * c - b * b;
    let t = (b * e - c * d) / denom;
    let s = (a * e - b * d) / denom;

    let p_closest = p1 + t * d1;
    let q_closest = p2 + s * d2;

    (p_closest + q_closest) / 2.//(p_closest, q_closest)
}