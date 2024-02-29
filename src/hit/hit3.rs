use std::{f32::{consts::PI, EPSILON, INFINITY}, hash::Hash};
use crate::{get_line_intersection3, log, CurveShape, FacetShape, Spatial3};

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
    pub curves:       (CurveShape, CurveShape),
    pub facets:       (FacetShape, FacetShape),
    pub spatial:      Spatial3,
    pub points:       Vec<Vec3>,
    pub step:         f32,
    pub tolerance:    f32,
}

pub struct Hit3 {
    pub hits:    (CurveShape, CurveShape),
    pub center_curve: CurveShape,
    pub start_point:  Vec3,
}

impl HitTester3 { 
    pub fn test(&mut self, start_uv0: Vec2, start_uv1: Vec2) -> Result<Hit3, (Miss, Miss)> { 
        let mut uv0 = start_uv0;
        let mut uv1 = start_uv1;
        let mut p0 = self.facets.0.get_point_at_uv(uv0);
        let mut p1 = self.facets.1.get_point_at_uv(uv1);
        let mut distance = INFINITY;
        let mut distance_basis = INFINITY;
        for _ in 0..20 {
            let center = self.get_tangent_intersection(uv0, uv1, p0, p1);
            let (uv0_t0, p0_t0) = self.facets.0.get_uv_and_point_from_target(uv0, center - p0);
            let (uv1_t0, p1_t0) = self.facets.1.get_uv_and_point_from_target(uv1, center - p1);
            let center = (p0 + p1) / 2.;
            let (uv0_t1, p0_t1) = self.facets.0.get_uv_and_point_from_target(uv0, center - p0);
            let (uv1_t1, p1_t1) = self.facets.1.get_uv_and_point_from_target(uv1, center - p1);
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
                    for control in hit.center_curve.controls.clone() {
                        self.spatial.insert(&control, self.points.len()); 
                        self.points.push(control);
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
        let normal0 = self.facets.0.get_normal_at_uv(uv0);
        let normal1 = self.facets.1.get_normal_at_uv(uv1);
        Err((
            Miss{dot:(p1 - p0).normalize().dot(normal1), distance}, 
            Miss{dot:(p0 - p1).normalize().dot(normal0), distance},
        ))
    }

    fn trace(&self, start: &HitPointUV) -> Option<Hit3> { 
        let start_point = (start.p0 + start.p1) / 2.;
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();
        let mut center_curve = CurveShape::default();
        let mut looped = false;
        let mut potential_duplicates = vec![];
        'direction_loop: for direction in 0..2 {
            let HitPointUV {mut uv0, mut uv1, mut p0, mut p1} = start;
            let mut controls0 = vec![];
            let mut controls1 = vec![];
            let mut center_controls = vec![];
            let mut add_points = |hit: HitPointUV| -> Vec3 {
                let center = (hit.p0 + hit.p1) / 2.;
                controls0.push(hit.uv0.extend(0.));
                controls1.push(hit.uv1.extend(0.));
                center_controls.push(center);
                center
            };
            'step_loop: for k in 0..1000 {
                let target = self.get_tangent_intersection(uv0, uv1, p0, p1);
                (uv0, p0) = self.facets.0.get_uv_and_point_from_target(uv0, target - p0);
                (uv1, p1) = self.facets.1.get_uv_and_point_from_target(uv1, target - p1);
                if k > 10 {//if total_steps > 10 {
                    if p0.distance(start.p0) < self.step || p1.distance(start.p1) < self.step {
                        add_points(start.clone());
                        looped = true;
                        break 'step_loop
                    }
                }
                let center = add_points(HitPointUV{uv0, uv1, p0, p1});
                let normal0 = self.facets.0.get_normal_at_uv(uv0);
                let normal1 = self.facets.1.get_normal_at_uv(uv1);
                let normal_cross = normal0.cross(normal1).normalize();
                let dir = normal_cross * (1-direction*2) as f32;
                for i in self.spatial.get(&center) {
                    let dist = self.points[i].distance(center);
                    if dist > self.step*0.01 && dist < self.step {
                        if (self.points[i]-center).normalize().dot(dir) > 0. {
                            potential_duplicates.push(i);
                        } 
                        if (self.points[i]-center).normalize().dot(dir) < 0.{
                            if potential_duplicates.contains(&i) {
                                return None
                            }
                        }
                    }
                }
                (uv0, p0) = self.facets.0.get_uv_and_point_from_target(uv0, dir * self.step);
                (uv1, p1) = self.facets.1.get_uv_and_point_from_target(uv1, dir * self.step);
                if uv0.x < EPSILON || uv0.x > 1.-EPSILON || uv0.y < EPSILON || uv0.y > 1.-EPSILON {
                    (uv1, p1) = self.facets.1.get_uv_and_point_from_target(uv1, p0 - p1);
                    add_points(HitPointUV{uv0, uv1, p0, p1});
                    break 'step_loop
                } 
                if uv1.x < EPSILON || uv1.x > 1.-EPSILON || uv1.y < EPSILON || uv1.y > 1.-EPSILON {
                    (uv0, p0) = self.facets.0.get_uv_and_point_from_target(uv0, p1 - p0);
                    add_points(HitPointUV{uv0, uv1, p0, p1});
                    break 'step_loop
                }
            }
            if center_controls.len() > 1 {
                if direction == 0 {
                    curve0.controls.extend(controls0);
                    controls1.reverse();
                    curve1.controls.splice(0..0, controls1);
                    center_curve.controls.extend(center_controls);
                }else{
                    controls0.reverse();
                    curve0.controls.splice(0..0, controls0);
                    curve1.controls.extend(controls1);
                    center_controls.reverse();
                    center_curve.controls.splice(0..0, center_controls);
                }
            }
            if looped {break 'direction_loop}
        }
        if center_curve.controls.is_empty() {
            return None
        }
        Some(Hit3{
            hits: (curve0.get_valid(), curve1.get_valid()),
            center_curve: center_curve.get_valid(),
            start_point,
        })
    }

    fn get_tangent_intersection(&self, uv0: Vec2, uv1: Vec2, p0: Vec3, p1: Vec3) -> Vec3 { 
        let normal0 = self.facets.0.get_normal_at_uv(uv0);
        let normal1 = self.facets.1.get_normal_at_uv(uv1);
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