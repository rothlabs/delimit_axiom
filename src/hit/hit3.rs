use std::{f32::{consts::PI, EPSILON}, hash::Hash};
use crate::{get_line_intersection3, log, nurbs::curve, CurveShape, FacetShape, Shape, Spatial3};

//use super::union3::UnionBasis3;
use rand::Rng;
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// #[derive(Clone)]
// pub struct CurveHit {
//     pub u: f32,
//     pub point: Vec3,
//     pub dot: f32,
// }

#[derive(Clone)]
pub struct FacetHit {
    pub uv0: Vec2,
    pub p0: Vec3,
    pub uv1: Vec2,
    pub p1: Vec3,
    //pub dot: f32,
}

//#[derive(Clone)]
pub struct HitTester3 {
    pub curves:       Vec<CurveShape>,
    pub facets:       Vec<FacetShape>,
    pub facet_index0: usize,
    pub facet_index1: usize,
    pub hit_map:      Vec<Spatial3>,
    pub hit_points:   Vec<Vec<Vec3>>,
    pub hit_step:     f32,
    pub tolerance:    f32,
}

pub struct Hit3 {
    pub curve0:       CurveShape,
    pub curve1:       CurveShape,
    pub center_curve: CurveShape,
    pub start_point0: Vec3,
    pub start_point1: Vec3,
}

impl HitTester3 { 
    pub fn test(&mut self, start_uv0: Vec2, start_uv1: Vec2) -> Option<Hit3> { 
        let facet0 = &self.facets[self.facet_index0];
        let facet1 = &self.facets[self.facet_index1];
        let mut uv0 = start_uv0;
        let mut uv1 = start_uv1;
        let mut p0 = facet0.get_point_at_uv(uv0);
        let mut p1 = facet1.get_point_at_uv(uv1);
        for _ in 0..10 {
            // (uv0, p0) = facet0.get_uv_and_point_from_3d_dir(uv0, p1 - p0);
            // (uv1, p1) = facet1.get_uv_and_point_from_3d_dir(uv1, p0 - p1);
            let center = self.get_center(uv0, uv1, p0, p1);
            let (uv0_t0, p0_t0) = facet0.get_uv_and_point_from_target(uv0, center - p0);
            let (uv1_t0, p1_t0) = facet1.get_uv_and_point_from_target(uv1, center - p1);
            let center = (p0 + p1) / 2.;
            let (uv0_t1, p0_t1) = facet0.get_uv_and_point_from_target(uv0, center - p0);
            let (uv1_t1, p1_t1) = facet1.get_uv_and_point_from_target(uv1, center - p1);
            if p0_t0.distance(p1_t0) < p0_t1.distance(p1_t1) {
                p0 = p0_t0;
                p1 = p1_t0;
                uv0 = uv0_t0;
                uv1 = uv1_t0;
            } else {
                p0 = p0_t1;
                p1 = p1_t1;
                uv0 = uv0_t1;
                uv1 = uv1_t1;
            }
            if p0.distance(p1) < self.tolerance * 0.5 {
                break;
            }
        }
        if p0.distance(p1) < self.tolerance {
            // if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON
            // && uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
            //     None
            // }else{
                // if self.hit_map[self.facet_index0].contains_key(&p0) && self.hit_map[self.facet_index1].contains_key(&p1) {
                //     None
                // } else {
                    let start = FacetHit {uv0, uv1, p0, p1};
                    let (curve0, curve1, curve2) = self.get_hit_curves(&start); // f0, f1, 
                    //self.hit_polylines[*f0].push(curve0.controls.iter().map(|v| v.truncate()).collect());
                    if curve0.controls.len() < 2 {
                        None
                    }else {
                        let first_point = curve0.controls.first().unwrap();
                        let last_point = curve0.controls.last().unwrap();
                        let mut duplicate_curve = false;
                        self.hit_map[self.facet_index0].for_pairs(&mut |i0: usize, i1: usize| {
                            if first_point.distance(self.hit_points[self.facet_index0][i0]) < self.tolerance {
                                if last_point.distance(self.hit_points[self.facet_index0][i1]) < self.tolerance {
                                    duplicate_curve = true;
                                }
                            }
                            if last_point.distance(self.hit_points[self.facet_index0][i0]) < self.tolerance {
                                if first_point.distance(self.hit_points[self.facet_index0][i1]) < self.tolerance {
                                    duplicate_curve = true;
                                }
                            }
                        });
                        if duplicate_curve {
                            None
                        } else {
                            self.hit_map[self.facet_index0].insert(first_point, self.hit_points[self.facet_index0].len());
                            self.hit_points[self.facet_index0].push(*first_point);
                            //self.hit_map[self.facet_index1].insert(&curve1.controls.first().unwrap(), 0);
                            self.hit_map[self.facet_index0].insert(last_point, self.hit_points[self.facet_index0].len());
                            self.hit_points[self.facet_index0].push(*last_point);
                            //self.hit_map[self.facet_index1].insert(&curve1.controls.last().unwrap(), 0);
                            
                            //self.shapes.push(Shape::Curve(curve0.get_valid()));
                            //self.shapes.push(Shape::Curve(curve1.get_valid()));
                            // self.shapes.push(Shape::Point(*curve2.controls.first().unwrap()));
                            // self.shapes.push(Shape::Point(*curve2.controls.last().unwrap()));
                            //self.shapes.push(Shape::Curve(curve2.get_valid()));
                            //Some((curve0.get_valid(), curve1.get_valid(), curve2.get_valid()))
                            Some(Hit3{
                                curve0: curve0.get_valid(),
                                curve1: curve1.get_valid(),
                                center_curve: curve2.get_valid(),
                                start_point0: p0,
                                start_point1: p1,
                            })
                        }
                    }
                //}
            //}
        }else{
            None
        }
    }

    fn get_hit_curves(&self, start: &FacetHit) -> (CurveShape, CurveShape, CurveShape) { 
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();
        let mut curve2 = CurveShape::default();
        // curve0.nurbs.order = 2;
        // curve1.nurbs.order = 2;
        let facet0 = &self.facets[self.facet_index0];
        let facet1 = &self.facets[self.facet_index1];
        let mut forward_controls0  = vec![]; 
        let mut forward_controls1  = vec![]; 
        let mut forward_controls2  = vec![]; 
        let mut backward_controls0 = vec![];
        let mut backward_controls1 = vec![];
        let mut backward_controls2 = vec![];
        'dir_loop: for direction in 0..2 {
            let FacetHit {mut uv0, mut uv1, mut p0, mut p1} = start;
            let mut add_points = |hit: FacetHit| {
                if direction == 0 {
                    forward_controls0.push(hit.uv0.extend(0.));
                    forward_controls1.push(hit.uv1.extend(0.));
                    forward_controls2.push((hit.p0 + hit.p1) / 2.);
                }else {
                    backward_controls0.push(hit.uv0.extend(0.));
                    backward_controls1.push(hit.uv1.extend(0.));
                    backward_controls2.push((hit.p0 + hit.p1) / 2.);
                } 
            };
            for k in 0..1000 {
                let center = self.get_center(uv0, uv1, p0, p1);
                (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, center - p0);
                (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, center - p1);
                if k > 14 {
                    if p0.distance(start.p0) < self.hit_step || p1.distance(start.p1) < self.hit_step {
                        add_points(start.clone());
                        break 'dir_loop;
                    }
                }
                add_points(FacetHit{uv0, uv1, p0, p1});
                
                let normal0 = facet0.get_normal_at_uv(uv0);
                let normal1 = facet1.get_normal_at_uv(uv1);
                let normal_cross = normal0.cross(normal1).normalize();
                let dir = normal_cross * (1-direction*2) as f32 * self.hit_step;
                (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, dir);
                (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, dir);
                //if k > 14 {
                    // if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON 
                    // && uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                    //     break;
                    // }
                    if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON {
                        //if p0.distance(p1) > self.tolerance {
                            (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, p0 - p1);
                        //}
                        add_points(FacetHit{uv0, uv1, p0, p1});
                        break;
                    } 
                    if uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                        //if p0.distance(p1) > self.tolerance {
                            (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, p1 - p0);
                        //}
                        add_points(FacetHit{uv0, uv1, p0, p1});
                        break;
                    }
                //}
            }
        }
        forward_controls0.reverse();
        curve0.controls.extend(forward_controls0);
        curve0.controls.extend(backward_controls0);

        backward_controls1.reverse();
        curve1.controls.extend(backward_controls1);
        curve1.controls.extend(forward_controls1);

        backward_controls2.reverse();
        curve2.controls.extend(backward_controls2);
        curve2.controls.extend(forward_controls2);

        (curve0, curve1, curve2)
    }

    fn get_center(&self, uv0: Vec2, uv1: Vec2, p0: Vec3, p1: Vec3) -> Vec3 { // facet0: FacetShape, facet1: FacetShape, 
        let normal0 = self.facets[self.facet_index0].get_normal_at_uv(uv0);
        let normal1 = self.facets[self.facet_index1].get_normal_at_uv(uv1);
        let normal_cross = normal0.cross(normal1).normalize();
        let cross0 = normal0.cross(normal_cross).normalize();
        let cross1 = normal1.cross(normal_cross).normalize();
        get_line_intersection3(p0, cross0, p1, cross1)
    }
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