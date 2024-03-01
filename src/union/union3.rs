use crate::{hit::Miss, log, nurbs::curve, CurveShape, FacetShape, HitTester3, Shape, Spatial3};
use glam::*;

use super::union2::UnionBasis2;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


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
    pub fn build(&mut self) -> (Vec<CurveShape>, Vec<FacetShape>) {
        self.test_groups();
        self.curves.extend(self.curve_groups[0].clone());
        self.curves.extend(self.curve_groups[1].clone());
        //self.facets.extend(self.facet_groups[0].clone());
        //self.facets.extend(self.facet_groups[1].clone());

        for g in 0..2 {
            //console_log!("facet group count: {}", self.facet_groups[g].len());
            for i in 0..self.facet_groups[g].len() {
                if self.facet_hits[g][i].is_empty() {
                    //self.miss[g][i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                    //if self.miss[g][i].is_empty() || self.miss[g][i][0].dot > -0.01 {
                        self.facets.push(self.facet_groups[g][i].clone());
                    //}
                }else{
                    // self.facet_hits[g][i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
                    self.add_bounded_facet(g, i);  
                }
            }
        }

        (self.curves.clone(), self.facets.clone())
        //let mut shapes = (vec![], vec![]); 
        // for i in 0..self.facets.len() {
        //     let mut facet = self.facets[i].clone();
        //     // if facet.boundaries.is_empty() {
        //     //     facet.perimeter = true;
        //     // }
        //     facet.boundaries.extend(self.facet_hits[i].clone());
        //     self.shapes.push(Shape::Facet(facet));
        // }
        // for i in 0..self.curves.len() {
        //     self.shapes.push(Shape::Curve(self.curves[i].clone()));
        // }
    }

    fn add_bounded_facet(&mut self, g: usize, i: usize) {
        let mut facet = self.facet_groups[g][i].clone();
        //console_log!("facet normal {}", facet.get_normal_at_uv(vec2(0.5, 0.5)));
        //let wow = facet.boundaries.iter().map(|c| Shape::Curve(c.clone()));
        //console_log!("face boundary count: {}", wow.len());
        //self.shapes.extend(wow);

        // let mut curves0 = vec![self.facet_hits[g][i][0].clone()];
        // for curves1 in self.facet_hits[g][i].iter().skip(1){
        //     let mut union = UnionBasis2::new(curves0.clone(), vec![curves1.clone()], 0.001);
        //     curves0 = union.build();
        // }
        let mut union = UnionBasis2::new(self.facet_hits[g][i].clone(), self.facet_hits[g][i].clone(), 0.001, true);
        let curves0 = union.build();
        // let mut curves0 = vec![];
        // for mut curve in union.build() {
        //     //curve.negate();
        //     curves0.push(curve);
        // }

        let mut union = UnionBasis2::new(facet.boundaries.clone(), curves0.clone(), 0.001, false); // self.facet_hits[g][i].clone()
        facet.boundaries = union.build();
        for mut boundary in facet.boundaries.clone() {
            for k in 0..boundary.controls.len() {
                //console_log!("facet i {}", i);
                boundary.controls[k] += vec3(100., g as f32 * 2., 0.);
                boundary.controls[k] += vec3(i as f32 * 2., 0., 0.);
                //boundary.controls[i] += Vec3::Y * g as f32 * 2.;
            }
            self.shapes.push(Shape::Curve(boundary));
        }

        //self.shapes.extend(facet.boundaries.iter().map(|c| Shape::Curve(c.clone())));
        //self.shapes.extend(union.shapes);
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

    fn test_groups(&mut self){
        for i0 in 0..self.facet_groups[0].len() {
            for i1 in 0..self.facet_groups[1].len() {
                self.tester.facets.0 = self.facet_groups[0][i0].clone();
                self.tester.facets.1 = self.facet_groups[1][i1].clone();
                self.tester.points = vec![];
                self.tester.spatial = Spatial3::new(self.tester.step);
                for uv0 in self.facet_groups[0][i0].get_normalized_knots() {
                    for uv1 in self.facet_groups[1][i1].get_normalized_knots() {
                        self.test_facets(i0, i1, uv0, uv1);
                    }
                }
            }
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