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
        //dir0 = dir0.rotate(Vec2::from_angle(self.rng.gen::<f32>()*PI*2.));
        //dir1 = dir1.rotate(Vec2::from_angle(self.rng.gen::<f32>()*PI*2.));
        let mut distance = p0.distance(p1);
        for i in 0..self.max_walk_iterations {
            if distance < self.tolerance { 
                break; 
            }
            // if i == self.max_walk_iterations-1 {
            //     log("Hit3 max iterations!");
            // }
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
            if uv0.x < self.tolerance || uv0.x > 1.-self.tolerance || uv0.y < self.tolerance || uv0.y > 1.-self.tolerance 
            || uv1.x < self.tolerance || uv1.x > 1.-self.tolerance || uv1.y < self.tolerance || uv1.y > 1.-self.tolerance {
                None
            }else{
                let first_hit = Facet_Hit {uv0, uv1, p0, p1, dot: 0.};
                let (curve0, curve1) = self.get_hit_curve(f0, f1, &first_hit);
                //self.hit_polylines[*f0].push(curve0.controls.iter().map(|v| v.truncate()).collect());
                if curve0.controls.len() > 2 {
                    self.shapes.push(Shape::Point(p0));
                    self.shapes.push(Shape::Point(p1));
                    self.shapes.push(Shape::Curve(curve0.get_valid()));
                    self.shapes.push(Shape::Curve(curve1.get_valid()));
                }
                Some(first_hit)
            }
        }else{
            None
        }
    }

    fn get_hit_curve(&self, f0: &usize, f1: &usize, first_hit: &Facet_Hit) -> (CurveShape, CurveShape) {
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();
        curve0.nurbs.order = 2;
        curve1.nurbs.order = 2;
        let facet0 = &self.facets[*f0];
        let facet1 = &self.facets[*f1];
        let mut forward_controls0 = vec![]; // first_hit.p0
        let mut backward_controls0 = vec![];
        let mut forward_controls1 = vec![]; // first_hit.p1
        let mut backward_controls1 = vec![];
        // // let mut hit_limit0 = false;
        // // let mut hit_limit1 = false;
        // let speed0 = self.facet_params.get(f0).unwrap().step.length() / 100.;
        // let speed1 = self.facet_params.get(f1).unwrap().step.length() / 100.;
        'dir_loop: for direction in 0..2 {
            //self.hit_map.insert(&uv0, endpoint_key);
            let Facet_Hit {mut uv0, mut p0, mut uv1, mut p1, dot} = first_hit;
            // let mut dir0 = Vec2::X; // self.hit_cell_size / 10.;
            // let mut dir1 = Vec2::X;
            'walk_loop: for k in 0..120 {
                let normal0 = facet0.get_normal_at_uv(uv0);
                let normal1 = facet1.get_normal_at_uv(uv1);
                let normal_cross = normal0.cross(normal1).normalize();
                let cross0 = normal0.cross(normal_cross).normalize();
                let cross1 = normal1.cross(normal_cross).normalize();
                let center = get_center_of_lines(p0, cross0, p1, cross1);
                /////let target = center + normal_cross * self.hit_cell_size * (1-direction*2) as f32 / 2.;
                //if p0.distance(p1) > self.tolerance {
                (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, center - p0);
                (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, center - p1);
                //}
                
                // if uv0.x < self.tolerance || uv0.x > 1.-self.tolerance || uv0.y < self.tolerance || uv0.y > 1.-self.tolerance {
                //     //(uv1, p1, dir1) = self.get_walked_point(facet1, p0, uv1, p1, dir1, speed1);
                //     break_walk = true;
                // } else if uv1.x < self.tolerance || uv1.x > 1.-self.tolerance || uv1.y < self.tolerance || uv1.y > 1.-self.tolerance {
                //     //(uv0, p0, dir0) = self.get_walked_point(facet0, p1, uv0, p0, dir0, speed0);
                //     break_walk = true;
                // }
                if direction == 0 {
                    forward_controls0.push(p0);
                    forward_controls1.push(p1);
                }else {
                    backward_controls0.push(p0);
                    backward_controls1.push(p1);
                } 
                
                let normal0 = facet0.get_normal_at_uv(uv0);
                let normal1 = facet1.get_normal_at_uv(uv1);
                let normal_cross = normal0.cross(normal1).normalize();
                let dir = normal_cross * self.hit_cell_size * (1-direction*2) as f32;
                (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, dir);
                (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, dir);

                let mut break_walk = false;
                if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON {
                    if p0.distance(p1) > 0.0001 {
                        (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, p0 - p1);
                    }
                    break_walk = true;
                } else if uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                    if p0.distance(p1) > 0.0001 {
                        (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, p1 - p0);
                    }
                    break_walk = true;
                }
                if break_walk {
                    if direction == 0 {
                        forward_controls0.push(p0);
                        forward_controls1.push(p1);
                    }else {
                        backward_controls0.push(p0);
                        backward_controls1.push(p1);
                    } 
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
            //     /////let target = center + normal_cross * self.hit_cell_size * (1-direction*2) as f32 / 2.;
            //     (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, center - p0);
            //     (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, center - p1);


            // fn get_walked_point(&self, facet0: &FacetShape, target: Vec3, uv0: Vec2, point: Vec3, dir: Vec2, speed: f32) -> (Vec2, Vec3, Vec2) {
            //     //let mut hit_limit = false;
            //     let mut uv = uv0;
            //     let mut point = point;
            //     let mut direction = Vec2::X;//dir * speed;//Vec2::X * self.hit_cell_size / 10.;
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