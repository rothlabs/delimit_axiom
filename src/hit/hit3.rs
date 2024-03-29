use std::{f32::{consts::PI, EPSILON, INFINITY}, hash::Hash};
use crate::{get_line_intersection3, log, nurbs::curve, CurveShape, FacetShape, Shape, Spatial3};

//use super::union3::UnionBasis3;
use rand::Rng;
use glam::*;

use super::Miss;

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
pub struct HitPointUV {
    pub uv0: Vec2,
    pub p0: Vec3,
    pub uv1: Vec2,
    pub p1: Vec3,
    //pub dot: f32,
}

//#[derive(Clone)]
pub struct HitTester3 {
    pub curve_groups: (Vec<CurveShape>, Vec<CurveShape>),
    pub facet_groups: (Vec<FacetShape>, Vec<FacetShape>),
    pub curve_index:  (usize, usize),
    pub facet_index:  (usize, usize),
    pub spatial:      Vec<Spatial3>,
    pub points:       Vec<Vec<Vec3>>,
    pub step:         f32,
    pub tolerance:    f32,
}

pub struct Hit3 {
    pub hits:    (Vec<CurveShape>, Vec<CurveShape>),
    pub center_curves: Vec<CurveShape>,
    pub start_point:  Vec3,
}

impl HitTester3 { 
    pub fn test(&mut self, start_uv0: Vec2, start_uv1: Vec2) -> Result<Hit3, (Miss, Miss)> { 
        let facet0 = &self.facet_groups.0[self.facet_index.0];
        let facet1 = &self.facet_groups.1[self.facet_index.1];
        let mut uv0 = start_uv0;
        let mut uv1 = start_uv1;
        let mut p0 = facet0.get_point_at_uv(uv0);
        let mut p1 = facet1.get_point_at_uv(uv1);
        let mut distance = INFINITY;
        let mut distance_basis = INFINITY;
        for _ in 0..20 {
            let center = self.get_tangent_intersection(uv0, uv1, p0, p1);
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
            distance = p0.distance(p1);
            if distance < self.tolerance {
                let start = HitPointUV {uv0, uv1, p0, p1};
                if let Some(hit) = self.trace(&start) {
                    for curve in &hit.center_curves {
                        for control in curve.controls.clone() {
                            self.spatial[self.facet_index.0].insert(&control, self.points[self.facet_index.0].len());
                            self.points[self.facet_index.0].push(control);
                        }
                    }
                    return Ok(hit);
                }
                break;
            }
            if distance >= distance_basis {//console_log!("break early! {}", i);
                break;
            }
            distance_basis = distance;
        }
        let normal0 = facet0.get_normal_at_uv(uv0);
        let normal1 = facet1.get_normal_at_uv(uv1);
        Err((
            Miss{dot:(p1 - p0).normalize().dot(normal1), distance}, 
            Miss{dot:(p0 - p1).normalize().dot(normal0), distance},
        ))
    }

    fn trace(&self, start: &HitPointUV) -> Option<Hit3> { 
        let start_point = (start.p0 + start.p1) / 2.;
        let facet0 = &self.facet_groups.0[self.facet_index.0];
        let facet1 = &self.facet_groups.1[self.facet_index.1];

        //let bdry_u0 = (0..facet0.boundaries.len()).map(||)

        let mut curves0 = vec![];
        let mut curves1 = vec![];
        let mut curves2 = vec![];
        let mut looped = false;
        //let mut duplicate = false;
        let mut potential_duplicates = vec![];
        //let mut center = Vec3::ZERO;
        'direction_loop: for direction in 0..2 {
            let HitPointUV {mut uv0, mut uv1, mut p0, mut p1} = start;
            //let mut out_of_rect = false;
            //let mut total_steps = 0;
            //'curve_loop: for _ in 0..100 {
                let mut curve0 = CurveShape::default();
                let mut curve1 = CurveShape::default();
                let mut curve2 = CurveShape::default();
                let mut add_points = |hit: HitPointUV| -> Vec3 {
                    let center = (hit.p0 + hit.p1) / 2.;
                    curve0.controls.push(hit.uv0.extend(0.));
                    curve1.controls.push(hit.uv1.extend(0.));
                    curve2.controls.push(center);
                    center
                };
                'step_loop: for k in 0..1000 {
                    let target = self.get_tangent_intersection(uv0, uv1, p0, p1);
                    (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, target - p0);
                    (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, target - p1);
                    if k > 10 {//if total_steps > 10 {
                        if p0.distance(start.p0) < self.step || p1.distance(start.p1) < self.step {
                            add_points(start.clone());
                            looped = true;
                            break
                        }
                    }
                    let center = add_points(HitPointUV{uv0, uv1, p0, p1});
                    let normal0 = facet0.get_normal_at_uv(uv0);
                    let normal1 = facet1.get_normal_at_uv(uv1);
                    let normal_cross = normal0.cross(normal1).normalize();
                    let dir = normal_cross * (1-direction*2) as f32;
                    for i in self.spatial[self.facet_index.0].get(&center) {
                        let dist = self.points[self.facet_index.0][i].distance(center);
                        if dist > self.step*0.01 && dist < self.step {
                            if (self.points[self.facet_index.0][i]-center).normalize().dot(dir) > 0. {
                                potential_duplicates.push(i);
                            } 
                            if (self.points[self.facet_index.0][i]-center).normalize().dot(dir) < 0.{
                                if potential_duplicates.contains(&i) {
                                    return None
                                }
                            }
                        }
                    }
                    (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, dir * self.step);
                    (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, dir * self.step);
                    if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON {
                        (uv1, p1) = facet1.get_uv_and_point_from_target(uv1, p0 - p1);
                        add_points(HitPointUV{uv0, uv1, p0, p1});
                        //out_of_rect = true;
                        break 'step_loop
                    } 
                    if uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                        (uv0, p0) = facet0.get_uv_and_point_from_target(uv0, p1 - p0);
                        add_points(HitPointUV{uv0, uv1, p0, p1});
                        //out_of_rect = true;
                        break 'step_loop
                    }

                    //total_steps += 1;
                }
                if curve0.controls.len() > 1 {
                    if direction == 0 {
                        curve0.controls.reverse();
                    }else{
                        curve1.controls.reverse();
                    }
                    curves0.push(curve0.get_valid());
                    curves1.push(curve1.get_valid());
                    curves2.push(curve2.get_valid());
                }
                //if out_of_rect {break 'curve_loop}
                if looped      {break 'direction_loop}
            //}
        }
        if curves0.is_empty() {
            return None
        }
        Some(Hit3{
            hits: (curves0, curves1),
            center_curves: curves2,
            start_point,
        })
    }

    fn get_tangent_intersection(&self, uv0: Vec2, uv1: Vec2, p0: Vec3, p1: Vec3) -> Vec3 { // facet0: FacetShape, facet1: FacetShape, 
        let normal0 = self.facet_groups.0[self.facet_index.0].get_normal_at_uv(uv0);
        let normal1 = self.facet_groups.1[self.facet_index.1].get_normal_at_uv(uv1);
        let normal_cross = normal0.cross(normal1).normalize();
        let cross0 = normal0.cross(normal_cross).normalize();
        let cross1 = normal1.cross(normal_cross).normalize();
        get_line_intersection3(p0, cross0, p1, cross1)
    }
}



// let first_point = curve0.controls.first().unwrap();
// let last_point = curve0.controls.last().unwrap();
// let mut duplicate = false;
// self.spatial[self.facet_index.0].for_pairs(&mut |i0: usize, i1: usize| {
//     if first_point.distance(self.points[self.facet_index.0][i0]) < self.tolerance {
//         if last_point.distance(self.points[self.facet_index.0][i1]) < self.tolerance {
//             duplicate = true;
//         }
//     }
//     if last_point.distance(self.points[self.facet_index.0][i0]) < self.tolerance {
//         if first_point.distance(self.points[self.facet_index.0][i1]) < self.tolerance {
//             duplicate = true;
//         }
//     }
// });
// if !duplicate {

    // self.spatial[self.facet_index.0].insert(first_point, self.points[self.facet_index.0].len());
    //                 self.points[self.facet_index.0].push(*first_point);
    //                 self.spatial[self.facet_index.0].insert(last_point, self.points[self.facet_index.0].len());
    //                 self.points[self.facet_index.0].push(*last_point);


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