use glam::*;
use crate::{log, HitBasis3};
use crate::{CurveShape, FacetShape, Shape, Trim};
use super::union2::UnionBasis2;

pub struct UnionBasis3 {
    pub hit_basis: HitBasis3,
    pub curve_groups: Vec<Vec<CurveShape>>,
    pub facet_groups: Vec<Vec<FacetShape>>,
    pub shapes: Vec<Shape>,
}

impl UnionBasis3 { 
    pub fn get_shapes(
        curve_groups: Vec<Vec<CurveShape>>, facet_groups: Vec<Vec<FacetShape>>,
    ) -> Vec<Shape> {
        UnionBasis3 {
            hit_basis: HitBasis3::new(facet_groups.clone()),
            curve_groups,
            facet_groups,
            shapes: vec![],
        }.make_shapes()
    }

    pub fn make_shapes(&mut self) -> Vec<Shape> {//-> (Vec<CurveShape>, Vec<FacetShape>) {
        //self.test_groups().unwrap(); //.expect("3D intersection failed");
        // self.curves.extend(self.curve_groups[0].clone());
        // self.curves.extend(self.curve_groups[1].clone());
        //let mut facet_indices: Vec<(usize, usize)> = vec![];
        self.hit_basis.make().expect("Facet intersection should succeed for union3 to work.");
        let hits = self.hit_basis.facet_hits.clone();
        let mut misses = self.hit_basis.facet_miss.clone();
        self.shapes = self.hit_basis.shapes.clone();
        for gi in 0..self.facet_groups.len() {
            for fi in 0..self.facet_groups[gi].len() {
                let mut collect_facet = false;
                for hi in 0..hits[gi][fi].len() {
                    if hits[gi][fi][hi].is_empty() {
                        //if !collect_facet {
                            // misses[gi][fi][hi] = misses[gi][fi][hi].clone().into_iter().filter(
                            //     |a| !a.distance.is_nan() && !a.dot.is_nan() && a.dot.abs() > 0.01
                            // ).collect();
                            misses[gi][fi][hi].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                            if misses[gi][fi][hi].is_empty() || misses[gi][fi][hi][0].dot * self.facet_groups[gi][fi].nurbs.sign < 0.01 {   
                                //self.facets.push(self.facet_groups[gi][fi].clone());
                                //self.shapes.push(Shape::Facet(self.facet_groups[gi][fi].clone()));
                                //facet_indices.push((gi, fi));
                                collect_facet = true;
                            }else{
                                collect_facet = false;
                                break; // This should ensure the facet is not collected in later hit groups
                            }
                        //}
                    }else{
                        self.union_facet_with_hits(gi, fi, hi, gi);  
                        //self.shapes.push(Shape::Facet(self.facet_groups[gi][fi].clone()));
                        //facet_indices.push((gi, fi));
                        collect_facet = true;
                    }
                }
                if collect_facet {
                    let mut facet = self.facet_groups[gi][fi].clone();
                    if facet.nurbs.sign < 0. {facet.reverse().negate();}
                    self.shapes.push(Shape::Facet(facet));
                }
            }
        }
        for curve_group in &self.curve_groups {
            for curve in curve_group {
                self.shapes.push(Shape::Curve(curve.clone()));
            }
        }
        // for (gi, fi) in facet_indices {
        //     let mut facet = self.facet_groups[gi][fi].clone();
        //     if facet.nurbs.sign < 0. {facet.reverse_normal().negate();}
        //     self.shapes.push(Shape::Facet(facet));
        // }
        self.shapes.clone()
        // for facet in &mut self.facets {
        //     if facet.nurbs.sign < 0. {facet.reverse_normal().negate();}
        // }
        //(self.curves.clone(), self.facets.clone())
    }

    fn union_facet_with_hits(&mut self, gi: usize, fi: usize, hi: usize, index: usize) {
        let facet = self.facet_groups[gi].get_mut(fi).expect("Should be a facet at this index.");
        //let mut facet = self.facet_groups[gi][fi].clone();
        if facet.nurbs.sign < 0. {
            for curve in &mut facet.boundaries {
                curve.negate();
            }
        }

        // for curve in &self.hit_basis.facet_hits[gi][fi][hi] {
        //     for control in curve.controls.windows(2) {
        //         if (control[0] - control[1]).length() < 0.00001 {
        //             log("bad facet hits!");
        //         }
        //     }
        // }

        let mut trim = Trim::new(self.hit_basis.facet_hits[gi][fi][hi].clone(), 0.008); // 0.001
        let curves1 = trim.build();

        // for curve in &curves1 {
        //     for control in curve.controls.windows(2) {
        //         if (control[0] - control[1]).length() < 0.00001 {
        //             log("bad trim");
        //         }
        //     }
        // }


        // for j in 0..facet.boundaries.len() {
        //     let mut bndry = facet.boundaries[j].clone();
        //     bndry.controls.clear();
        //     for k in 0..facet.boundaries[j].controls.len() {
        //         bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
        //             100. + fi as f32 * 2.,// + (j as f32)*0.005,  
        //             gi as f32 * 2.,// + (j as f32)*0.01, 
        //             0.
        //         ));
        //     }
        //     self.shapes.push(Shape::Curve(bndry));
        // }


        let mut union = UnionBasis2::new(facet.boundaries.clone(), curves1.clone(), 0.008, false); // self.facet_hits[g][i].clone()
        facet.boundaries = union.build();


        // for curve in &facet.boundaries {
        //     for control in curve.controls.windows(2) {
        //         if (control[0] - control[1]).length() < 0.00001 {
        //             log("bad UnionBasis2");
        //         }
        //     }
        // }

                // for shape in union.shapes {
                //     if let Shape::Point(point) = shape {
                //         self.shapes.push(Shape::Point(point));
                //     }
                // }
        //if index < 2 {
            // for j in 0..self.hit_basis.facet_hits[gi][fi][hi].len() {
            //     let mut bndry = self.hit_basis.facet_hits[gi][fi][hi][j].clone();
            //     bndry.controls.clear();
            //     for k in 0..self.hit_basis.facet_hits[gi][fi][hi][j].controls.len() {
            //         bndry.controls.push(self.hit_basis.facet_hits[gi][fi][hi][j].controls[k] + vec3(
            //             100. + fi as f32 * 2., // + (j as f32)*0.01,  
            //             gi as f32 * 2., //  + (j as f32)*0.01, 
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
            //             100. + fi as f32 * 2., //  + (j as f32)*0.01  
            //             gi as f32 * 2., //  + (j as f32)*0.01 
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
                        100. + fi as f32 * 2.,// + (j as f32)*0.005,  
                        gi as f32 * 2.,// + (j as f32)*0.01, 
                        0.
                    ));
                }
                self.shapes.push(Shape::Curve(bndry));
            }
        //}
        //self.facets.push(facet);
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


    // fn draw_center(&self, y: i32) {
    //     self.gpu.gl.use_program(Some(&self.center_program));
    //     self.gpu.set_uniform_1i(&self.center_program, "source_tex",  3);
    //     self.gpu.set_uniform_2i(&self.center_program, "viewport_position",  IVec2::Y*y);
    //     self.gpu.draw_rect(&self.trace_buffer.as_ref().unwrap().center, IVec2::Y*y, IVec2::Y);
    // }

// let hits = self.get_hit_points();
        // for hit in hits {
        //     let mut curve0 = CurveShape::default();
        //     curve0.negate();
        //     for point in hit.points0 {
        //         curve0.controls.push(vec3(point[0], point[1], 0.));
        //     }
        //     if !curve0.controls.is_empty() {
        //         self.facet_hits[hit.g0][hit.f0].push(curve0.get_valid());
        //         //self.shapes.push(Shape::Curve(curve0.get_valid()));
        //     }
        //     let mut curve1 = CurveShape::default();
        //     curve1.negate();
        //     for point in hit.points1 {
        //         curve1.controls.push(vec3(point[0], point[1], 0.));
        //     }
        //     if !curve1.controls.is_empty() {
        //         self.facet_hits[hit.g1][hit.f1].push(curve1.get_valid());
        //     }
        //     let mut center_curve = CurveShape::default();
        //     center_curve.negate();
        //     for point in hit.centers {
        //         center_curve.controls.push(Vec3::from_array(point));
        //     }
        //     if !center_curve.controls.is_empty() {
        //         //curve0.remove_doubles(self.tester.tolerance * 2.);
        //         //self.facet_hits[hit.g0][hit.f0].push(curve0.get_valid());
        //         self.shapes.push(Shape::Curve(center_curve.get_valid()));
        //     }
        // } 

// let point_buf0 = self.gpu.framebuffer.make_empty_rgba32f(2, point_buf_size0)?;
        // let uv_buf0 = self.gpu.framebuffer.make_rgba32f(3, &mut pack.uv_texels)?;
        // let uv_buf1 = self.gpu.framebuffer.make_empty_rgba32f(2, pair_buf_size0)?;

// let max_facet_loc = self.gpu.gl.get_uniform_location(&self.point_program, "max_facet_length");
        // self.gpu.gl.uniform1i(max_facet_loc.as_ref(), self.buffer.as_ref().unwrap().max_facet_length);

// fn get_hit_points(&self) -> Vec<FacetHit> { 
//     let mut facet_groups = vec![];
//     for group in &self.facet_groups { 
//         let mut facet_group = FacetGroup::default();
//         for facet in group {
//             facet_group.facets.push(Facet{
//                 sign:    facet.nurbs.sign,
//                 order:   facet.nurbs.order,
//                 knots:   facet.nurbs.knots.clone(),
//                 weights: facet.nurbs.weights.clone(),
//                 controls:   facet.controls.iter().map(|c| Model::Curve(Curve{
//                     sign: c.nurbs.sign,
//                     order: c.nurbs.order,
//                     knots: c.nurbs.knots.clone(),
//                     weights: c.nurbs.weights.clone(),
//                     controls: c.controls.iter().map(|v| Model::Point(v.to_array())).collect(),
//                     min: c.min,
//                     max: c.max,
//                 })).collect(),
//                 boundaries: vec![],
//             });
//         }
//         facet_groups.push(serde_wasm_bindgen::to_value(&facet_group).unwrap());
//     }
//     let mut hits = vec![];
//     let js_values = get_facet_hit_points(facet_groups);
//     for js_value in js_values {
//         hits.push(serde_wasm_bindgen::from_value(js_value).unwrap());
//     }
//     hits
// }

            // tester: HitTester3 {
            //     curves: (CurveShape::default(), CurveShape::default()),
            //     facets: (FacetShape::default(), FacetShape::default()),
            //     spatial: Spatial3::new(step), // (0..facets.len()).map(|_| Spatial3::new(step)).collect(), // 
            //     points:  vec![],
            //     tolerance,
            //     step,
            //     hone_count: 8,
            // },

// fn test_facets(&mut self, i0: usize, i1: usize, uv0: Vec2, uv1: Vec2) { // facet_index0: usize, facet_index1: usize, 
    //     match self.tester.test(uv0, uv1) {
    //         Ok(hit) => {
    //             self.facet_hits[0][i0].push(hit.hits.0);
    //             self.facet_hits[1][i1].push(hit.hits.1);
    //             self.shapes.push(Shape::Curve(hit.center_curve));
    //             self.shapes.push(Shape::Point(hit.start_point));
    //         },
    //         Err(miss) => {
    //             self.facet_miss[0][i0].push(miss.0);
    //             self.facet_miss[1][i1].push(miss.1);
    //         }
    //     }
    // }


        // for hit in hits {
        //     //if hit.g0 != g0 || hit.g1 != g1 || hit.f0 != f0 || hit.f1 != f1 {
        //         g0 = hit.g0;
        //         g1 = hit.g1;
        //         f0 = hit.f0;
        //         f1 = hit.f1;
        //         self.tester.facets.0 = self.facet_groups[g0][f0].clone();
        //         self.tester.facets.1 = self.facet_groups[g1][f1].clone();
        //     //     self.tester.spatial = Spatial3::new(self.tester.step);
        //     // }
        //     self.test_facets(hit.f0, hit.f1, Vec2::from_array(hit.uv0), Vec2::from_array(hit.uv1));
        //     let point = self.facet_groups[hit.g0][hit.f0].get_point_at_uv(Vec2::from_array(hit.uv0));
        //     self.shapes.push(Shape::Point(point));
        // }

        // for i0 in 0..self.facet_groups[0].len() {
        //     for i1 in 0..self.facet_groups[1].len() {
        //         self.tester.facets.0 = self.facet_groups[0][i0].clone();
        //         self.tester.facets.1 = self.facet_groups[1][i1].clone();
        //         self.tester.points = vec![];
        //         self.tester.spatial = Spatial3::new(self.tester.step);
        //         for uv0 in self.facet_groups[0][i0].get_normalized_knots() {
        //             for uv1 in self.facet_groups[1][i1].get_normalized_knots() {
        //                 self.test_facets(i0, i1, uv0, uv1);
        //             }
        //         }
        //     }
        // }


        // //get_facet_hit_points(facet_groups[0].clone(), facet_groups[1].clone(), max_controls[0]*max_controls[1]);
        // //let boxes1: Vec<euclid::Box3D<f32, f32>> = self.facet_groups[1].iter().map(|fct| fct.get_box3()).collect();
        // // let mut box1 = Box3D::zero();
        // // for facet in &self.facet_groups[1] {
        // //     box1 = box1.union(&facet.get_box3());
        // // }
        // for i0 in 0..self.facet_groups[0].len() {
        //     //let box0 = self.facet_groups[0][i0].get_box3();
        //     //Box3D::union(&self, other)
        //     //if box0.intersects(&box1) {


// let mut facet_groups = vec![];
//         let mut max_controls = vec![0, 0];
//         for g in 0..2 {
//             let mut facet_group = FacetGroup::default();
//             for facet in &self.facet_groups[g] {
//                 max_controls[g] = max_controls[g].max(facet.controls.iter().fold(0 as usize, |a,b| a + b.controls.len()));
//                 let new_facet = Facet{
//                     sign: facet.nurbs.sign,
//                     order:   facet.nurbs.order,
//                     knots:   facet.nurbs.knots.clone(),
//                     weights: facet.nurbs.weights.clone(),
//                     controls:   facet.controls.iter().map(|c| Model::Curve(Curve{
//                         sign: c.nurbs.sign,
//                         order: c.nurbs.order,
//                         knots: c.nurbs.knots.clone(),
//                         weights: c.nurbs.weights.clone(),
//                         controls: c.controls.iter().map(|v| Model::Point(v.to_array())).collect(),
//                         min: c.min,
//                         max: c.max,
//                     })).collect(),
//                     boundaries: vec![],
//                 };
//                 facet_group.facets.push(new_facet);
//             }
//             facet_groups.push(serde_wasm_bindgen::to_value(&facet_group).unwrap());
//         }
//         get_facet_hit_points(facet_groups.clone());




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