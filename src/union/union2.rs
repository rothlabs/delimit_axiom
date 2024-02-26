//use std::{collections::HashMap, f32::EPSILON};
use crate::{log, nurbs::curve, CurveHit, CurveMiss, CurveShape, Hit2, HitTester2, Shape};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub struct UnionBasis2 {
    pub tester:  HitTester2,
    pub curves:  Vec<CurveShape>,
    pub grouped: Vec<Vec<CurveShape>>,
    pub hits:    Vec<Vec<CurveHit>>, 
    pub miss:    Vec<Vec<CurveMiss>>, 
    pub shapes:  Vec<Shape>,
}

impl UnionBasis2 { 
    pub fn get_shapes(&mut self) -> Vec<Shape> {
        self.test_pairs();
        for i in 0..self.curves.len() {
            if self.hits[i].is_empty() {
                self.miss[i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                if self.miss[i].first().unwrap().dot > 0.1 {
                    self.shapes.push(Shape::Curve(self.curves[i].clone()));
                }
                continue;
            }
            self.hits[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
            self.add_split_curves(i);
        }
        self.shapes.clone()
    }

    fn test_hit(&mut self, u0: f32, u1: f32) { 
        match self.tester.test(u0, u1) {
            Ok(hit) => {
                self.hits[self.tester.index0].push(hit.hit0);
                self.hits[self.tester.index1].push(hit.hit1);
                self.shapes.push(Shape::Point(hit.center));
            },
            Err(hit) => {
                self.miss[self.tester.index0].push(hit.miss0);
                self.miss[self.tester.index1].push(hit.miss1);
            }
        }
    }

    fn test_pairs(&mut self){
        let mut ac0 = 0;
        for cg0 in 0..self.grouped.len() {
            let mut ac1 = ac0;
            for cg1 in cg0..self.grouped.len() {
                if cg0 != cg1 { 
                    for c0 in 0..self.grouped[cg0].len() {
                        for c1 in 0..self.grouped[cg1].len() {
                            self.tester.index0 = ac0 + c0;
                            self.tester.index1 = ac1 + c1;
                            if self.tester.index0 == self.tester.index1 {
                                log("tried to use same curve indecies!!!");
                                continue;
                            }
                            for u0 in self.curves[self.tester.index0].get_normalized_knots() {
                                for u1 in self.curves[self.tester.index1].get_normalized_knots() {
                                    self.test_hit(u0, u1);
                                }
                            }
                        }
                    }
                }
                ac1 += self.grouped[cg1].len();
            }
            ac0 += self.grouped[cg0].len();
        }
    }

    fn add_split_curves(&mut self, i: usize) {
        let first = self.hits[i].first().unwrap();
        let mut set_min = false;
        if first.dot > 0. {set_min = true;}
        let mut curve = self.curves[i].clone();
        for curve_hit in &self.hits[i] { 
            if set_min {
                curve.min = curve_hit.u;
            }else{
                curve.max = curve_hit.u;
                self.shapes.push(Shape::Curve(curve));
                curve = self.curves[i].clone();
            }
            set_min = !set_min;
        }
        if !set_min {
            self.shapes.push(Shape::Curve(curve));
        }
    }
}




        //console_log!("try face pairs: {}, {}", self.grouped_facets.len(), self.grouped_facets.len());
        //let start = Instant::now();
                //let elapsed = start.elapsed();
        //console_log!("timed: {:?}", elapsed);




// pub fn get_shapes(&mut self) -> Vec<Shape> {
//     let spatial = self.set_samples_and_get_spatial();
//     self.clear_params();
//     self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_curve_param);
//     self.reduce_cell_and_step();
//     let spatial = self.set_samples_and_get_spatial();
//     self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_intersection);
//     for i in 0..self.curves.len() {
//         self.hits[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
//         if self.hits[i].is_empty() {
//             self.shapes.push(Shape::Curve(self.curves[i].clone()));
//             continue;
//         }
//         self.add_split_curves(i);
//     }
//     self.shapes.clone()
// }

// fn add_split_curves(&mut self, i: usize) {
//     let first = self.hits[i].first().unwrap();
//     let mut set_min = false;
//     if first.angle > 0. {set_min = true;}
//     let mut curve = self.curves[i].clone();
//     for itc in self.get_merged_hits(i, first) { 
//         self.shapes.push(Shape::Point(vec3(itc.point.x, itc.point.y, 0.)));
//         if set_min {
//             curve.min = itc.u;
//         }else{
//             curve.max = itc.u;
//             self.shapes.push(Shape::Curve(curve));
//             curve = self.curves[i].clone();
//         }
//         set_min = !set_min;
//     }
//     if !set_min {
//         self.shapes.push(Shape::Curve(curve));
//     }
// }

// fn get_merged_hits(&self, i: usize, first: &Hit2) -> Vec<Hit2> {
//     let mut point = first.point;
//     let mut intersections = vec![first.clone()];
//     for itc in &self.hits[i] {
//         if itc.point.distance(point) > self.cell_size {
//             intersections.push(itc.clone());
//         }
//         point = itc.point;
//     }
//     intersections
// }

// fn clear_params(&mut self) {
//     for i in 0..self.curves.len() {
//         if let Some(cr) = self.curve_params.get_mut(&i) {
//             cr.params.clear();
//         }
//     }
// }

// fn add_curve_param(&mut self, curve_index0: usize, _c1: usize, u0: f32, _u1: f32) {
//     if let Some(cr) = self.curve_params.get_mut(&curve_index0) {
//         cr.params.push(u0);
//     }
// }

// fn add_intersection(&mut self, curve_index0: usize, curve_index1: usize, u0: f32, u1: f32) {
//     if let Some(itc) = self.get_hit(&curve_index0, &curve_index1, u0, u1) {
//         if 0.01 < itc.u && itc.u < 0.99 {
//             self.hits[curve_index0].push(itc.clone());
//         } 
//     }
// }

// fn for_spatial_pairs<F>(&mut self, spatial: &Spatial2, func: &mut F) 
// where F: FnMut(&mut UnionBasis2, usize, usize, f32, f32)  { 
//     spatial.for_pairs(&mut |i0: usize, i1: usize| { 
//         let Sample2 {index: c0, point: p0, u: u0} = self.samples[i0];
//         let Sample2 {index: c1, point: p1, u: u1} = self.samples[i1];
//         if c0 == c1 {return}
//         if p0.distance(p1) > self.cell_size {return}
//         func(self, c0, c1, u0, u1);
//     });
// }


// fn reduce_cell_and_step(&mut self) {
//     for i in 0..self.curves.len() {
//         if let Some(cr) = self.curve_params.get_mut(&i) {
//             cr.params.sort_by(|a, b| a.partial_cmp(b).unwrap());
//             if cr.params.is_empty() {continue}
//             let mut filled = vec![cr.params[0]];
//             for uu in cr.params.windows(2) {
//                 if uu[1] - uu[0] <= cr.step + EPSILON {
//                     for k in 1..20 {
//                         let fill_u = uu[0] + k as f32 * (cr.step/10.);
//                         if fill_u >= uu[1] {break}
//                         filled.push(fill_u);
//                     }
//                 }
//                 filled.push(uu[1]);
//             }
//             cr.params = filled;
//             cr.step /= 10.
//         }
//     }
//     self.cell_size /= 10.;
// }


// fn set_samples_and_get_spatial(&mut self) -> Spatial2 { 
//     let mut spatial: Spatial2 = Spatial2::new(self.cell_size); 
//     self.samples.clear();
//     for (_, CurveParams {i, params, ..}) in &self.curve_params { 
//         for u in params {
//             let point = self.curves[*i].get_vec2_at_u(*u);
//             self.samples.push(Sample2 {
//                 index: *i,
//                 point,
//                 u: *u,
//             });
//             spatial.insert(&point, self.samples.len()-1);
//         }
//     }
//     spatial
// }



//for_merged_intersections(&self.intersections[i].clone(), self.cell_size, &mut |itc: &Intersection2| { 

// fn for_merged_intersections<F>(intersections: &Vec<Intersection2>, tolerance: f32, func: &mut F)// -> Vec<Intersection2> 
// where F: FnMut(&Intersection2)  {
//     let mut point = intersections.first().unwrap().point;
//     //let mut intersections = vec![first.clone()];
//     for itc in intersections{
//         if itc.point.distance(point) > tolerance {
//             func(&itc);
//             //intersections.push(itc.clone());
//         }
//         point = itc.point;
//     }
//     //intersections
// }