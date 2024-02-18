use std::{f32::{consts::PI, EPSILON}, hash::Hash};

use crate::{log, CurveShape, FacetShape, Shape};


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
            let first_hit = Facet_Hit {
                uv0,
                point0: p0,
                uv1,
                point1: p1,
                dot: 0.,
            };
            //let curve = self.get_hit_curve(f0, f1, &hit);
            
            let endpoint_key = (p0.x * 92837111. + p0.y * 689287499. + p0.z * 283923481.).floor() as usize;
            let curve_key = endpoint_key + 1;
            let mut curve = CurveShape::default();
            //let first_hit = 
            //curve.controls.push(first_hit.point0);
            let mut forward_controls = vec![first_hit.point0];
            let mut backward_controls = vec![];
            'dir_loop: for direction in 0..2 {
                self.hit_map.insert(&p0, endpoint_key);
                let Facet_Hit {mut uv0, point0: mut p0, mut uv1, point1: mut p1, dot} = first_hit;
                'walk_loop: for k in 0..120 {
                    let normal0 = facet0.get_normal_at_uv(uv0);
                    let normal1 = facet1.get_normal_at_uv(uv1);
                    let normal_cross = normal0.cross(normal1);
                    let cross0 = normal0.cross(normal_cross);
                    let cross1 = normal1.cross(normal_cross);
                    let center = approx_line_intersection(p0, cross0, p1, cross1);
                    let target = center + normal_cross * self.hit_cell_size * (1-direction*2) as f32;
                    let mut dir = Vec2::X * self.hit_cell_size / 10.;
                    let mut distance = p0.distance(target);
                    for i in 0..self.max_walk_iterations {
                        if distance < self.tolerance { 
                            break; 
                        }
                        if i == self.max_walk_iterations-1 {
                            log("get_hit_curve max iterations!");
                        }
                        uv0 = (uv0 + dir).clamp(Vec2::ZERO, Vec2::ONE);
                        p0 = facet0.get_point_at_uv(uv0);
                        let dist = p0.distance(target);
                        if dist >= distance {
                            dir = dir.perp() * 0.8;
                        }
                        distance = dist;
                        if let Some(keys) = self.hit_map.get(&p0) {
                            // if k > 8 {
                            //     if keys.contains(&endpoint_key) {break 'outer}
                            // } 
                            // for key in keys {
                            //     if key != &curve_key {
                            //         //curve.controls.push(first_hit.point0);
                            //         break 'walk_loop;
                            //     }
                            // }
                        }
                    }
                    // if p0.x < EPSILON || p0.x > 1.-EPSILON || p0.y < EPSILON || p0.y > 1.-EPSILON {
                    //     curve.controls.push(p0);
                    //     break;
                    // }

                    
                    
                    dir = Vec2::X * self.hit_cell_size / 10.;
                    distance = p1.distance(target);
                    for i in 0..self.max_walk_iterations {
                        if distance < self.tolerance { 
                            break; 
                        }
                        if i == self.max_walk_iterations-1 {
                            log("get_hit_curve max iterations!");
                        }
                        uv1 = (uv1 + dir).clamp(Vec2::ZERO, Vec2::ONE);
                        p1 = facet1.get_point_at_uv(uv1);
                        let dist = p1.distance(target);
                        if dist >= distance {
                            dir = dir.perp() * 0.8;
                        }
                        distance = dist;
                    }
                    if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON 
                    || uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                        self.hit_map.insert(&p0, curve_key);
                        //curve.controls.push(p0);
                        break;
                    }
                    
                    //if k > 2 { // wait until out of start cell
                        
                    //    curve_key = (p0.x * 92837111. + p0.y * 689287499. + p0.z * 283923481.).floor() as usize;
                    //    self.hit_map.insert(&p0, curve_key);
                    //}
                    self.hit_map.insert(&p0, curve_key);
                    //curve.controls.push(p0);
                    if direction == 0 {
                        forward_controls.push(p0);
                    }else {
                        backward_controls.push(p0);
                    } 
                }
            }
            backward_controls.reverse();
            curve.controls.extend(backward_controls);
            curve.controls.extend(forward_controls);
            if curve.controls.len() > 2 {
                curve.nurbs.order = 3;
                self.shapes.push(Shape::Curve(curve.get_valid()));
                self.shapes.push(Shape::Point(first_hit.point0));
                self.shapes.push(Shape::Point(p0));
            }
            Some(first_hit)
        }else{
            None
        }
    }

    // fn get_hit_curve(&self, f0: &usize, f1: &usize, hit: &Facet_Hit) -> CurveShape {
    //     let mut curve = CurveShape::default();
    //     let facet0 = &self.facets[*f0];
    //     let facet1 = &self.facets[*f1];
    //     let Facet_Hit {mut uv0, mut uv1, point0: mut p0, point1: mut p1, dot} = hit.clone();
        
    //}
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


            //let delta = 0.0001;
            // let normal0 = facet0.get_normal_at_uv(uv0);
            // let normal1 = facet1.get_normal_at_uv(uv1);
            // let normal_cross = normal0.cross(normal1);
            // let cross0 = normal0.cross(normal_cross);
            // let cross1 = normal1.cross(normal_cross);
            // let center = approx_line_intersection(p0, cross0, p1, cross1);