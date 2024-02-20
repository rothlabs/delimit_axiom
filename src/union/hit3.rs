use std::{f32::{consts::PI, EPSILON}, hash::Hash};
use crate::{log, nurbs::curve, CurveShape, FacetShape, Shape};

use super::union3::UnionBasis3;
use rand::Rng;
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone)]
pub struct Curve_Hit {
    pub u: f32,
    pub point: Vec3,
    pub dot: f32,
}

#[derive(Clone)]
pub struct Facet_Hit {
    pub uv0: Vec2,
    pub p0: Vec3,
    pub uv1: Vec2,
    pub p1: Vec3,
    pub dot: f32,
}

impl UnionBasis3 { 
    pub fn start_hit_curves(&mut self, start0: Vec2, start1: Vec2) -> Option<Facet_Hit> { // f0: &usize, f1: &usize, 
        let facet0 = &self.facets[self.facet_index0];
        let facet1 = &self.facets[self.facet_index1];
        let mut uv0 = start0;
        let mut uv1 = start1;
        let mut p0 = facet0.get_point_at_uv(uv0);
        let mut p1 = facet1.get_point_at_uv(uv1);
        let center = self.get_center(uv0, uv1, p0, p1);
        (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, center - p0);
        (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, center - p1);
        if p0.distance(p1) < self.tolerance {
            if uv0.x < self.tolerance || uv0.x > 1.-self.tolerance || uv0.y < self.tolerance || uv0.y > 1.-self.tolerance 
            || uv1.x < self.tolerance || uv1.x > 1.-self.tolerance || uv1.y < self.tolerance || uv1.y > 1.-self.tolerance {
                None
            }else{
                if self.hit_map.contains_key(&p0) {
                    None
                } else {
                    let start = Facet_Hit {uv0, uv1, p0, p1, dot: 0.};
                    let (curve0, curve1) = self.make_hit_curves(&start); // f0, f1, 
                    //self.hit_polylines[*f0].push(curve0.controls.iter().map(|v| v.truncate()).collect());
                    if curve0.controls.len() > 2 {
                        self.shapes.push(Shape::Point(p0));
                        self.shapes.push(Shape::Point(p1));
                        self.shapes.push(Shape::Curve(curve0.get_valid()));
                        self.shapes.push(Shape::Curve(curve1.get_valid()));
                    }
                    Some(start)
                }
            }
        }else{
            None
        }
    }

    fn make_hit_curves(&mut self, start: &Facet_Hit) -> (CurveShape, CurveShape) { 
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();
        // curve0.nurbs.order = 2;
        // curve1.nurbs.order = 2;
        let facet0 = &self.facets[self.facet_index0];
        let facet1 = &self.facets[self.facet_index1];
        let mut forward_controls0  = vec![]; // first_hit.p0
        let mut backward_controls0 = vec![];
        let mut forward_controls1  = vec![]; // first_hit.p1
        let mut backward_controls1 = vec![];
        'dir_loop: for direction in 0..2 {
            let Facet_Hit {mut uv0, mut p0, mut uv1, mut p1, dot} = start;
            let mut add_points = |pt0: Vec3, pt1: Vec3| {
                if direction == 0 {
                    forward_controls0.push(pt0);
                    forward_controls1.push(pt1);
                }else {
                    backward_controls0.push(pt0);
                    backward_controls1.push(pt1);
                } 
            };
            for k in 0..10000 {
                let center = self.get_center(uv0, uv1, p0, p1);
                (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, center - p0);
                (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, center - p1);
                if k > 14 {
                    if p0.distance(start.p0) < self.hit_step * 2. || p1.distance(start.p1) < self.hit_step * 2. {
                        add_points(start.p0, start.p1);
                        break 'dir_loop;
                    }
                }
                add_points(p0, p1);
                self.hit_map.insert(&p0, 0);
                let normal0 = facet0.get_normal_at_uv(uv0);
                let normal1 = facet1.get_normal_at_uv(uv1);
                let normal_cross = normal0.cross(normal1).normalize();
                let dir = normal_cross * (1-direction*2) as f32 * self.hit_step;
                (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, dir);
                (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, dir);
                if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON {
                    //if p0.distance(p1) > self.tolerance {
                        (_, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, p0 - p1);
                    //}
                    add_points(p0, p1);
                    break;
                } else if uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                    //if p0.distance(p1) > self.tolerance {
                        (_, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, p1 - p0);
                    //}
                    add_points(p0, p1);
                    break;
                }
            }
        }
        backward_controls0.reverse();
        backward_controls1.reverse();
        curve0.controls.extend(backward_controls0);
        curve1.controls.extend(backward_controls1);
        curve0.controls.extend(forward_controls0);
        curve1.controls.extend(forward_controls1);
        (curve0, curve1)
    }

    fn get_center(&self, uv0: Vec2, uv1: Vec2, p0: Vec3, p1: Vec3) -> Vec3 { // facet0: FacetShape, facet1: FacetShape, 
        let normal0 = self.facets[self.facet_index0].get_normal_at_uv(uv0);
        let normal1 = self.facets[self.facet_index1].get_normal_at_uv(uv1);
        let normal_cross = normal0.cross(normal1).normalize();
        let cross0 = normal0.cross(normal_cross).normalize();
        let cross1 = normal1.cross(normal_cross).normalize();
        get_center_of_lines(p0, cross0, p1, cross1)
    }
}

fn get_center_of_lines(
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


// if p0.distance(p1) > self.tolerance {
//     (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, (p1 - p0) / 2.);
//     (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, (p0 - p1) / 2.);
// }

            //let delta = 0.0001;
            // let normal0 = facet0.get_normal_at_uv(uv0);
            // let normal1 = facet1.get_normal_at_uv(uv1);
            // let normal_cross = normal0.cross(normal1);
            // let cross0 = normal0.cross(normal_cross);
            // let cross1 = normal1.cross(normal_cross);
            // let center = approx_line_intersection(p0, cross0, p1, cross1);



            // let normal0 = facet0.get_normal_at_uv(uv0);
            //     let normal1 = facet1.get_normal_at_uv(uv1);
            //     let normal_cross = normal0.cross(normal1).normalize();
            //     let cross0 = normal0.cross(normal_cross).normalize();
            //     let cross1 = normal1.cross(normal_cross).normalize();
            //     let center = approx_line_intersection(p0, cross0, p1, cross1);
            //     /////let target = center + normal_cross * self.hit_step * (1-direction*2) as f32 / 2.;
            //     (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, center - p0);
            //     (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, center - p1);


            // fn get_walked_point(&self, facet0: &FacetShape, target: Vec3, uv0: Vec2, point: Vec3, dir: Vec2, speed: f32) -> (Vec2, Vec3, Vec2) {
            //     //let mut hit_limit = false;
            //     let mut uv = uv0;
            //     let mut point = point;
            //     let mut direction = Vec2::X;//dir * speed;//Vec2::X * self.hit_step / 10.;
            //     let mut distance = point.distance(target);
            //     for i in 0..100 {
            //         let dir = Vec2::from_angle((i as f32 / 99 as f32) * PI * 2.) * speed / 10.;
            //         let dist = facet0.get_point_at_uv(uv + dir).distance(target);
            //         if dist < distance {
            //             distance = dist;
            //             direction = dir;
            //         }
            //     }
            //     direction = direction.normalize() * speed;
            //     //console_log!("dir: {}, {}", direction.x, direction.y);
            //     distance = point.distance(target);
            //     for i in 0..self.max_walk_iterations {
            //         if distance < self.tolerance { //  || dir.length() < 0.0001 
            //             break; 
            //         }
            //         if i == self.max_walk_iterations-1 {
            //             log("get_hit_curve max iterations!");
            //         }
            //         uv = (uv + direction).clamp(Vec2::ZERO, Vec2::ONE);
            //         point = facet0.get_point_at_uv(uv);
            //         let dist = point.distance(target);
            //         if dist >= distance {
            //             direction = direction.perp() * 0.98;
            //             //console_log!("walk i: {}", i);
            //             //break;
            //         }
            //         distance = dist;
            //         // if uv.x < EPSILON || uv.x > 1.-EPSILON || uv.y < EPSILON || uv.y > 1.-EPSILON {
            //         //     //(uv1, p1) = self.get_walked_point(facet1, p0, uv1, p1);
            //         //     hit_limit = true;
            //         //     break;//break_walk = true;
            //         // } 
            //     }
            //     direction = (uv - uv0).normalize();
            //     (uv, point, direction)
            // }