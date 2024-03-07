use std::f32::EPSILON;

use crate::{hit::Miss, log, Circle, CurveShape, FacetShape, HitTester3, Shape, Spatial3, Trim};
use euclid::{box3d, Box3D};
use glam::*;
use super::union2::UnionBasis2;

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

//#[derive(Clone, Default)]
pub struct UnionBasis3 {
    pub tester: HitTester3,
    pub curve_groups: [Vec<CurveShape>; 2],
    pub facet_groups: [Vec<FacetShape>; 2],
    pub facet_hits: [Vec<Vec<CurveShape>>; 2], 
    pub facet_miss: [Vec<Vec<Miss>>; 2], 
    pub curves: Vec<CurveShape>,
    pub facets: Vec<FacetShape>,
    pub shapes: Vec<Shape>,
    //pub curve_hits: Vec<Vec<CurveHit>>,
}

impl UnionBasis3 { 
    pub fn new(curves0: Vec<CurveShape>, curves1: Vec<CurveShape>,
               facets0: Vec<FacetShape>, facets1: Vec<FacetShape>, tolerance: f32, step: f32) -> Self {
        UnionBasis3 {
            tester: HitTester3 {
                curves: (CurveShape::default(), CurveShape::default()),
                facets: (FacetShape::default(), FacetShape::default()),
                spatial: Spatial3::new(step), // (0..facets.len()).map(|_| Spatial3::new(step)).collect(), // 
                points:  vec![],
                tolerance,
                step,
                hone_count: 8,
            },
            facet_hits: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]], 
            facet_miss: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]],
            curve_groups: [curves0, curves1],
            facet_groups: [facets0, facets1],
            curves: vec![],
            facets: vec![],
            shapes: vec![],
        }
    }

    pub fn build(&mut self, index: usize) -> (Vec<CurveShape>, Vec<FacetShape>) {
        self.test_groups();
        self.curves.extend(self.curve_groups[0].clone());
        self.curves.extend(self.curve_groups[1].clone());
        for g in 0..2 {
            for i in 0..self.facet_groups[g].len() {
                if self.facet_hits[g][i].is_empty() {
                    self.facet_miss[g][i] = self.facet_miss[g][i].clone().into_iter().filter(
                        |a| !a.distance.is_nan() && !a.dot.is_nan() && a.dot.abs() > 0.01
                    ).collect();
                    self.facet_miss[g][i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                    if self.facet_miss[g][i].is_empty() || self.facet_miss[g][i][0].dot > -0.01 {
                        self.facets.push(self.facet_groups[g][i].clone());
                    }
                }else{
                    self.add_bounded_facet(g, i, index);  
                }
            }
        }
        for facet in &mut self.facets {
            if facet.nurbs.sign < 0. {facet.reverse_normal().negate();}
        }
        (self.curves.clone(), self.facets.clone())
    }

    fn add_bounded_facet(&mut self, g: usize, i: usize, index: usize) {
        let mut facet = self.facet_groups[g][i].clone();
        if facet.nurbs.sign < 0. {
            for curve in &mut facet.boundaries {
                curve.negate();
            }
        }
        // let mut curves0 = vec![];
        // for mut curve in facet.boundaries.clone() {
        //     if facet.nurbs.sign < 0. {
        //         curve.negate();
        //     }
        //     curves0.push(curve);
        // }

        

        let mut union = Trim::new(self.facet_hits[g][i].clone(), 0.001);
        let curves1 = union.build();
        //let mut curves1 = self.facet_hits[g][i].clone();
        // let circle = Circle{center:[0.2, 0.2], radius:0.05, reverse:true}.get_shapes();
        // if let Shape::Curve(mut circle) = circle[0].clone() {
        //     circle.negate();
        //     curves1 = vec![circle];//curves1.push(circle);
        // }

        // if index > 1 {
        //     for j in 0..facet.boundaries.len() {
        //         let mut bndry = facet.boundaries[j].clone();
        //         bndry.controls.clear();
        //         for k in 0..facet.boundaries[j].controls.len() {
        //             bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
        //                 100. + i as f32 * 2., //  + (j as f32)*0.01  
        //                 g as f32 * 2., //  + (j as f32)*0.01 
        //                 0.
        //             ));
        //         }
        //         self.shapes.push(Shape::Curve(bndry));
        //     }
        // }
        
        let mut union = UnionBasis2::new(facet.boundaries, curves1.clone(), 0.001, false); // self.facet_hits[g][i].clone()
        facet.boundaries = union.build();
        
        // for shape in union.shapes {
        //     if let Shape::Point(point) = shape {
        //         self.shapes.push(Shape::Point(point));
        //     }
        // }

        if index > 1 {
            // for j in 0..self.facet_hits[g][i].len() {
            //     let mut bndry = self.facet_hits[g][i][j].clone();
            //     bndry.controls.clear();
            //     for k in 0..self.facet_hits[g][i][j].controls.len() {
            //         bndry.controls.push(self.facet_hits[g][i][j].controls[k] + vec3(
            //             100. + i as f32 * 2., //  + (j as f32)*0.01  
            //             g as f32 * 2., //  + (j as f32)*0.01 
            //             0.
            //         ));
            //     }
            //     self.shapes.push(Shape::Curve(bndry));
            // }
            // for j in 0..curves1.len() {
            //     let mut bndry = curves1[j].clone();
            //     bndry.controls.clear();
            //     for k in 0..curves1[j].controls.len() {
            //         bndry.controls.push(curves1[j].controls[k] + vec3(
            //             100. + i as f32 * 2., //  + (j as f32)*0.01  
            //             g as f32 * 2., //  + (j as f32)*0.01 
            //             0.
            //         ));
            //     }
            //     self.shapes.push(Shape::Curve(bndry));
            // }
            for j in 0..facet.boundaries.len() {
                let mut bndry = facet.boundaries[j].clone();
                bndry.controls.clear();
                for k in 0..facet.boundaries[j].controls.len() {
                    bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
                        100. + i as f32 * 2. + (j as f32)*0.005,  
                        g as f32 * 2. + (j as f32)*0.01, 
                        0.
                    ));
                }
                self.shapes.push(Shape::Curve(bndry));
            }
        }

        
        self.facets.push(facet);
    }

    fn test_facets(&mut self, i0: usize, i1: usize, uv0: Vec2, uv1: Vec2) { // facet_index0: usize, facet_index1: usize, 
        match self.tester.test(uv0, uv1) {
            Ok(hit) => {
                self.facet_hits[0][i0].push(hit.hits.0);
                self.facet_hits[1][i1].push(hit.hits.1);
                self.shapes.push(Shape::Curve(hit.center_curve));
                self.shapes.push(Shape::Point(hit.start_point));
            },
            Err(miss) => {
                self.facet_miss[0][i0].push(miss.0);
                self.facet_miss[1][i1].push(miss.1);
            }
        }
    }

    fn test_groups(&mut self) {
        //let boxes1: Vec<euclid::Box3D<f32, f32>> = self.facet_groups[1].iter().map(|fct| fct.get_box3()).collect();
        // let mut box1 = Box3D::zero();
        // for facet in &self.facet_groups[1] {
        //     box1 = box1.union(&facet.get_box3());
        // }
        for i0 in 0..self.facet_groups[0].len() {
            //let box0 = self.facet_groups[0][i0].get_box3();
            //Box3D::union(&self, other)
            //if box0.intersects(&box1) {
                for i1 in 0..self.facet_groups[1].len() {
                    //if box0.intersects(&boxes1[i1]) {
                        self.tester.facets.0 = self.facet_groups[0][i0].clone();
                        self.tester.facets.1 = self.facet_groups[1][i1].clone();
                        self.tester.points = vec![];
                        self.tester.spatial = Spatial3::new(self.tester.step);
                        for uv0 in self.facet_groups[0][i0].get_normalized_knots() {
                            for uv1 in self.facet_groups[1][i1].get_normalized_knots() {
                                self.test_facets(i0, i1, uv0, uv1);
                            }
                        }
                    //}
                }
            //}
        }        
    }
}

//use std::time::Instant;
// use rand::{Rng, SeedableRng};
// use rand::rngs::StdRng;
//let seed: [u8; 32] = *b"seed_value_0123456789seed_value_";
//self.rng = SmallRng::from_seed(seed);

//console_log!("try face pairs: {}, {}", self.grouped_facets.len(), self.grouped_facets.len());
//let start = Instant::now();
//let elapsed = start.elapsed();
//console_log!("timed: {:?}", elapsed);






// let mut af0 = 0;
        // for fg0 in 0..self.facet_groups.len() {
        //     let mut af1 = 0;
        //     for fg1 in fg0..self.facet_groups.len() {
        //         if fg0 != fg1 {
        //             for f0 in 0..self.facet_groups[fg0].len() {
        //                 for f1 in 0..self.facet_groups[fg1].len() {
        //                     self.tester.facet_index.0 = af0 + f0;
        //                     self.tester.facet_index.1 = af1 + f1;
        //                     if self.tester.facet_index.0 == self.tester.facet_index.1 {
        //                         log("tried to use same facet indecies!!!");
        //                         continue;
        //                     }
        //                     self.test_facets(Vec2::ONE*0.5, Vec2::ONE*0.5);
        //                     for x in 0..2 {
        //                         for y in 0..2 {
        //                             for x1 in 0..2 {
        //                                 for y1 in 0..2 {
        //                                     self.test_facets(vec2(x as f32, y as f32), vec2(x1 as f32, y1 as f32));
        //                                 }
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         af1 += self.facet_groups[fg1].len();
        //     }
        //     af0 += self.facet_groups[fg0].len();
        // }


        // //console_log!("UnionBasis3 get_shapes");
        // let spatial = self.set_samples_and_get_spatial();
        // self.clear_params();
        // self.for_spatial_pairs(&spatial, &mut UnionBasis3::add_curve_param, &mut UnionBasis3::add_facet_hit);

    // fn for_spatial_pairs<C, F>(&mut self, spatial: &Spatial3, curve_func: &mut C, facet_func: &mut F) 
    // where C: FnMut(&mut UnionBasis3, usize, usize, f32, Vec2), 
    //       F: FnMut(&mut UnionBasis3, Vec2, Vec2) { // usize, usize, 
    //     let mut stop = false;
    //     spatial.for_pairs(&mut |i0: usize, i1: usize| { 
    //         if i1 < self.curve_samples.len() {return} // second index must be for facet_params
    //         let FacetSample {index: f1, point: p1, uv: uv1} = self.facet_samples[i1 - self.curve_samples.len()];
    //         if i0 < self.curve_samples.len() {
    //             let CurveSample {index: c0, point: p0, u: u0} = self.curve_samples[i0];
    //             //if c0 == f1 {return}
    //             //if p0.distance(p1) > self.cell_size {return}
    //             //curve_func(self, c0, f1, u0, uv1);
    //             //self.shapes.push(Shape::Point(p0));
    //         }else{
    //             if stop {return}
    //             let FacetSample {index: f0, point: p0, uv: uv0} = self.facet_samples[i0 - self.curve_samples.len()];
    //             //console_log!("facet: {}, {}, {}", p0.x, p0.y, p0.z);
    //             if f0 == f1 {return}
    //             if p0.distance(p1) > self.cell_size {return}
    //             self.facet_index0 = f0;
    //             self.facet_index1 = f1;
    //             facet_func(self, uv0, uv1); // f0, f1, 
    //             //stop = true;
    //             //self.shapes.push(Shape::Point(p0));
    //         }
    //     });
    // }

    // fn set_samples_and_get_spatial(&mut self) -> Spatial3 { 
    //     let mut spatial: Spatial3 = Spatial3::new(self.cell_size); 
    //     self.curve_samples.clear();
    //     for (_, CurveParams {i, params, ..}) in &self.curve_params { 
    //         for u in params {
    //             let point = self.curves[*i].get_point_at_u(*u);
    //             self.curve_samples.push(CurveSample {
    //                 index: *i,
    //                 point,
    //                 u: *u,
    //             });
    //             spatial.insert(&point, self.curve_samples.len()-1);
    //         }
    //     }
    //     self.facet_samples.clear();
    //     for (_, FacetParams {i, params, ..}) in &self.facet_params { 
    //         for uv in params {
    //             let point = self.facets[*i].get_point_at_uv(*uv);
    //             self.facet_samples.push(FacetSample {
    //                 index: *i,
    //                 point,
    //                 uv: *uv,
    //             });
    //             spatial.insert(&point, self.curve_samples.len() + self.facet_samples.len() - 1);
    //         }
    //     }
    //     spatial
    // }

    // fn clear_params(&mut self) {
    //     for i in 0..self.curves.len() {
    //         if let Some(cr) = self.curve_params.get_mut(&i) {
    //             cr.params.clear();
    //         }
    //     }
    //     for i in 0..self.facets.len() {
    //         if let Some(fr) = self.facet_params.get_mut(&i) {
    //             fr.params.clear();
    //         }
    //     }
    // }



// fn try_facet_pairs(&mut self){
//     for f0 in 0..self.facets.len() {
//         for f1 in 0..self.facets.len() {
//             if f0 == f1 {continue}
//             // self.facet_index0 = f0;
//             // self.facet_index1 = f1;
//             self.add_facet_hit(Vec2::ONE*0.5, Vec2::ONE*0.5);
//             for x in 0..2 {
//                 for y in 0..2 {
//                     for x1 in 0..2 {
//                         for y1 in 0..2 {
//                             self.add_facet_hit(vec2(x as f32, y as f32), vec2(x1 as f32, y1 as f32));
//                         }
//                     }
//                 }
//             }
//             // self.add_facet_hit(Vec2::ZERO, Vec2::ZERO);
//             // self.add_facet_hit(Vec2::ONE, Vec2::ONE);
//         }
//     }
// }