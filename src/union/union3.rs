use std::{collections::HashMap, f32::EPSILON};
use crate::{log, CurveShape, FacetShape, Shape, Spatial2, Spatial3};
use super::{hit3::{CurveHit, FacetHit}, CurveParams, FacetParams};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub struct CurveSample {
    index: usize,
    point: Vec3,
    u: f32,
}

pub struct FacetSample {
    index: usize,
    point: Vec3,
    uv:    Vec2,
}

//#[derive(Clone, Default)]
pub struct UnionBasis3 {
    pub facet0: FacetShape,
    pub facet1: FacetShape,
    pub facet_index0: usize,
    pub facet_index1: usize,
    //pub rng: StdRng,
    pub hit_map: Spatial3,
    pub curves: Vec<CurveShape>,
    pub facets: Vec<FacetShape>,
    pub grouped_curves: Vec<Vec<CurveShape>>,
    pub grouped_facets: Vec<Vec<FacetShape>>,
    pub curve_params: HashMap<usize, CurveParams>, 
    pub facet_params: HashMap<usize, FacetParams>, 
    pub cell_size: f32,
    pub hit_step: f32,
    pub shapes: Vec<Shape>,
    pub facet_hits: Vec<Vec<CurveShape>>,
    pub curve_hits: Vec<Vec<CurveHit>>,
    pub tolerance: f32,
    pub max_walk_iterations: usize,
    pub curve_samples: Vec<CurveSample>,
    pub facet_samples: Vec<FacetSample>,
}

impl UnionBasis3 { 
    pub fn get_shapes(&mut self) -> Vec<Shape> {
        //let seed: [u8; 32] = *b"seed_value_0123456789seed_value_";
        //self.rng = SmallRng::from_seed(seed);
        // //console_log!("UnionBasis3 get_shapes");
        // let spatial = self.set_samples_and_get_spatial();
        // self.clear_params();
        // self.for_spatial_pairs(&spatial, &mut UnionBasis3::add_curve_param, &mut UnionBasis3::add_facet_hit);
        self.try_facet_pairs();
        for i in 0..self.facets.len() {
            let mut facet = self.facets[i].clone();
            if facet.boundaries.is_empty() {
                facet.perimeter = true;
            }
            facet.boundaries.extend(self.facet_hits[i].clone());
            self.shapes.push(Shape::Facet(facet));
        }
        for i in 0..self.curves.len() {
            self.shapes.push(Shape::Curve(self.curves[i].clone()));
        }
        self.shapes.clone()
    }

    // fn add_curve_param(&mut self, curve_index0: usize, _f1: usize, u0: f32, _uv1: Vec2) {
    //     if let Some(cr) = self.curve_params.get_mut(&curve_index0) {
    //         cr.params.push(u0);
    //     }
    // }

    // fn add_facet_param(&mut self, facet_index0: usize, _f1: usize, uv0: Vec2, _uv1: Vec2) {
    //     if let Some(cr) = self.facet_params.get_mut(&facet_index0) {
    //         cr.params.push(uv0);
    //     }
    // }

    fn add_facet_hit(&mut self, uv0: Vec2, uv1: Vec2) { // facet_index0: usize, facet_index1: usize, 
        if let Some(hit) = self.try_facet_hit(uv0, uv1) { // &facet_index0, &facet_index1, 
            self.facet_hits[self.facet_index0].push(hit.0.clone());
            self.facet_hits[self.facet_index1].push(hit.1.clone());
        }
    }

    fn try_facet_pairs(&mut self){
        let mut af0 = 0;
        for fg0 in 0..self.grouped_facets.len() {
            let mut af1 = 0;
            for fg1 in 0..self.grouped_facets.len() {
                if fg0 != fg1 {
                    for f0 in 0..self.grouped_facets[fg0].len() {
                        for f1 in 0..self.grouped_facets[fg1].len() {
                            self.facet_index0 = af0 + f0;
                            self.facet_index1 = af1 + f1;
                            if self.facet_index0 == self.facet_index1 {
                                log("tried to use same facet indecies!!!");
                                continue;
                            }
                            self.add_facet_hit(Vec2::ONE*0.5, Vec2::ONE*0.5);
                            for x in 0..2 {
                                for y in 0..2 {
                                    for x1 in 0..2 {
                                        for y1 in 0..2 {
                                            self.add_facet_hit(vec2(x as f32, y as f32), vec2(x1 as f32, y1 as f32));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                af1 += self.grouped_facets[fg1].len();
            }
            af0 += self.grouped_facets[fg0].len();
        }
    }

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
}


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