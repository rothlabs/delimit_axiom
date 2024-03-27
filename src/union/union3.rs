use crate::gpu::framebuffer::Framebuffer;
use crate::union::texel::{IndexPair, TraceTexelBasis};
use crate::{get_facet_hit_points, hit::Miss, log, Curve, CurveShape, Facet, FacetGroup, FacetShape, Model, Shape, Trim};
use crate::gpu::{shader::COPY_FRAGMENT_SOURCE, GPU};
use glam::*;
use serde::{Deserialize, Serialize};
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram};
use super::traced::{get_traced_curves, TracedCurve};
use super::union2::UnionBasis2;
use super::texel::TexelBasis;
use super::shader::{CENTER_SOURCE, POINT_SOURCE, BOX_SOURCE, HIT_MISS_SOURCE, HONE_SOURCE, HONE_TRACE_SOURCE, TRACE_SOURCE};
use wasm_bindgen::JsValue;
use js_sys::Array;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "FacetHitPoint::default")]
pub struct FacetHitPoint {
    g0: usize,
    g1: usize,
    f0: usize,
    f1: usize,
    uv0: [f32; 2],
    uv1: [f32; 2],
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "FacetHit::default")]
pub struct FacetHit {
    g0: usize,
    g1: usize,
    f0: usize,
    f1: usize,
    points0: Vec<[f32; 2]>,
    points1: Vec<[f32; 2]>,
    centers: Vec<[f32; 3]>,
}

struct BasisBuffer {
    point: Framebuffer,
    uv0:   Framebuffer,
    uv1:   Framebuffer,
}

struct TraceBuffer {
    point:  Framebuffer,
    segment:    Framebuffer,
    honed:    Framebuffer,
    trace:  Framebuffer,
    //center: Framebuffer,
}

//#[derive(Clone, Default)]
pub struct UnionBasis3 {
    //pub tester: HitTester3,
    pub curve_groups: [Vec<CurveShape>; 2],
    pub facet_groups: [Vec<FacetShape>; 2],
    pub facet_hits: [Vec<Vec<CurveShape>>; 2], 
    pub facet_miss: [Vec<Vec<Miss>>; 2], 
    pub curves: Vec<CurveShape>,
    pub facets: Vec<FacetShape>,
    pub shapes: Vec<Shape>,
    //pub curve_hits: Vec<Vec<CurveHit>>,
    gpu: GPU,
    texel: TexelBasis,
    point_program:    WebGlProgram,   
    hone_program:     WebGlProgram,
    hit_miss_program: WebGlProgram, 
    copy_program:     WebGlProgram,
    trace_program:    WebGlProgram, 
    hone_trace_program:    WebGlProgram, 
    center_program:   WebGlProgram,
    box_program:      WebGlProgram,
    buffer:        Option<BasisBuffer>,
    trace_buffer:  Option<TraceBuffer>,
}

impl UnionBasis3 { 
    pub fn new(curves0: Vec<CurveShape>, curves1: Vec<CurveShape>,
               facets0: Vec<FacetShape>, facets1: Vec<FacetShape>, tolerance: f32, step: f32) -> Self {
        let facet_hits = [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]];
        let facet_miss = [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]];
        let facet_groups = [facets0, facets1];
        let gpu = GPU::new().unwrap();
        //let copy_program = gpu.get_quad_program_from_source(COPY_FRAGMENT_SOURCE).unwrap();
        UnionBasis3 {
            facet_hits,//: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]], 
            facet_miss,//: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]],
            curve_groups: [curves0, curves1],
            facet_groups,//: [facets0, facets1],
            curves: vec![],
            facets: vec![],
            shapes: vec![],
            texel: TexelBasis::default(),
            point_program:      gpu.get_quad_program_from_source(POINT_SOURCE).unwrap(),
            hone_program:       gpu.get_quad_program_from_source(HONE_SOURCE).unwrap(),
            hit_miss_program:   gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            copy_program:       gpu.get_quad_program_from_source(COPY_FRAGMENT_SOURCE).unwrap(),
            trace_program:      gpu.get_quad_program_from_source(TRACE_SOURCE).unwrap(),
            hone_trace_program: gpu.get_quad_program_from_source(HONE_TRACE_SOURCE).unwrap(),
            center_program:     gpu.get_quad_program_from_source(CENTER_SOURCE).unwrap(),
            box_program:        gpu.get_quad_program_from_source(BOX_SOURCE).unwrap(),
            buffer: None,
            trace_buffer: None,
            gpu,
        }
    }

    pub fn build(&mut self, index: usize) -> (Vec<CurveShape>, Vec<FacetShape>) {
        self.test_groups().unwrap(); //.expect("3D intersection failed");
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

        if index < 2 {
            for j in 0..self.facet_hits[g][i].len() {
                let mut bndry = self.facet_hits[g][i][j].clone();
                bndry.controls.clear();
                for k in 0..self.facet_hits[g][i][j].controls.len() {
                    bndry.controls.push(self.facet_hits[g][i][j].controls[k] + vec3(
                        100. + i as f32 * 2., //  + (j as f32)*0.01  
                        g as f32 * 2., //  + (j as f32)*0.01 
                        0.
                    ));
                }
                self.shapes.push(Shape::Curve(bndry));
            }
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
            // for j in 0..facet.boundaries.len() {
            //     let mut bndry = facet.boundaries[j].clone();
            //     bndry.controls.clear();
            //     for k in 0..facet.boundaries[j].controls.len() {
            //         bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
            //             100. + i as f32 * 2.,// + (j as f32)*0.005,  
            //             g as f32 * 2.,// + (j as f32)*0.01, 
            //             0.
            //         ));
            //     }
            //     self.shapes.push(Shape::Curve(bndry));
            // }
        }
        self.facets.push(facet);
    }

    fn set_uniform_basis(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "max_facet_length", self.texel.max_facet_length);
        self.gpu.set_uniform_1i(program, "max_knot_count",   self.texel.max_knot_count);
        self.gpu.set_uniform_1i(program, "facet_tex", 0);
        self.gpu.set_uniform_1i(program, "pair_tex",  1);
    }

    fn set_hone_program(&self, uv_i: i32) {
        self.gpu.gl.use_program(Some(&self.hone_program));
        self.set_uniform_basis(&self.hone_program);
        self.gpu.set_uniform_1i(&self.hone_program, "point_tex",  2);
        self.gpu.set_uniform_1i(&self.hone_program, "uv_tex",  uv_i);
    }

    fn draw_points(&self, uv_i: i32) {
        self.gpu.gl.use_program(Some(&self.point_program));
        self.set_uniform_basis(&self.point_program);
        self.gpu.set_uniform_1i(&self.point_program, "uv_tex",  uv_i);
        self.gpu.draw(&self.buffer.as_ref().unwrap().point)
    }

    fn hone_to_hit_or_miss(&self) {
        let buff = &self.buffer.as_ref().unwrap();
        for _ in 0..6 {
            self.draw_points(3);
            self.set_hone_program(3);
            self.gpu.draw(&buff.uv1);
            self.draw_points(4);
            self.set_hone_program(4);
            self.gpu.draw(&buff.uv0);
        }
        self.gpu.gl.use_program(Some(&self.hit_miss_program));
        self.set_uniform_basis(&self.hit_miss_program);
        self.gpu.set_uniform_1i(&self.hit_miss_program, "point_tex",  2);
        self.gpu.set_uniform_1i(&self.hit_miss_program, "uv_tex",  3);
        self.gpu.draw(&buff.uv1);
    }

    // fn draw_center(&self, y: i32) {
    //     self.gpu.gl.use_program(Some(&self.center_program));
    //     self.gpu.set_uniform_1i(&self.center_program, "source_tex",  3);
    //     self.gpu.set_uniform_2i(&self.center_program, "viewport_position",  IVec2::Y*y);
    //     self.gpu.draw_rect(&self.trace_buffer.as_ref().unwrap().center, IVec2::Y*y, IVec2::Y);
    // }

    fn draw_trace_points(&self, uv_i: i32) {
        self.gpu.gl.use_program(Some(&self.point_program));
        self.set_uniform_basis(&self.point_program);
        self.gpu.set_uniform_1i(&self.point_program, "uv_tex",  uv_i);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().point)
    }

    fn hone_trace(&self) {
        self.gpu.gl.use_program(Some(&self.hone_trace_program));
        self.set_uniform_basis(&self.hone_trace_program);
        self.gpu.set_uniform_1i(&self.hone_trace_program, "point_tex", 2);
        self.gpu.set_uniform_1i(&self.hone_trace_program, "uv_tex",  3);
        self.gpu.set_uniform_1i(&self.hone_trace_program, "box_tex", 4);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().honed);
    }

    fn copy_trace(&self, y: i32) {
        self.gpu.gl.use_program(Some(&self.copy_program));
        self.gpu.set_uniform_1i(&self.copy_program, "source_tex0",  5);
        self.gpu.set_uniform_1i(&self.copy_program, "source_tex1",  7);
        self.gpu.set_uniform_2i(&self.copy_program, "viewport_position",  IVec2::Y*y);
        self.gpu.draw_at_pos(&self.trace_buffer.as_ref().unwrap().trace, IVec2::Y*y);
    }

    fn trace_segment(&self) {
        self.gpu.gl.use_program(Some(&self.trace_program));
        self.set_uniform_basis(&self.trace_program);
        self.gpu.set_uniform_1i(&self.trace_program, "point_tex", 2);
        self.gpu.set_uniform_1i(&self.trace_program, "uv_tex",  5);
        self.gpu.set_uniform_1i(&self.trace_program, "box_tex", 6);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().segment);
    }

    fn trace(&self, length: i32){
        for y in 0..length {
            self.draw_trace_points(3);
            self.hone_trace();
            self.copy_trace(y);
            self.draw_trace_points(5);
            self.trace_segment();
        }
    }

    fn test_groups(&mut self) -> Result<(), String> {
        let mut basis0 = TexelBasis::new(self.facet_groups.to_vec());
        self.gpu.texture.make_r32f(0, &mut basis0.facet_texels)?;
        let (_, pair_buf_size0) = self.gpu.texture.make_rg32i(1, &mut basis0.pair_texels)?;
        let point_buf_size0 = ivec2(pair_buf_size0.x*3, pair_buf_size0.y*2);
        self.buffer = Some(BasisBuffer{
            point: self.gpu.framebuffer.make_empty_rgba32f(2, point_buf_size0)?,
            uv0:   self.gpu.framebuffer.make_rgba32f(3, &mut basis0.uv_texels)?,
            uv1:   self.gpu.framebuffer.make_empty_rgba32f(4, pair_buf_size0)?,
        });
        self.texel = basis0;
        self.hone_to_hit_or_miss();
        let buff0 = &self.buffer.as_ref().unwrap();
        let hit_miss = self.gpu.read(&buff0.uv1, 0);
        // for i in 0..hit_miss.len()/4 {
        //     if hit_miss[i*4] > -0.5 {
        //         let IndexPair{g0, g1, f0, f1} = self.texel.index_pairs[i];
        //         let point = self.facet_groups[g0][f0].get_point_at_uv(vec2(hit_miss[i*4], hit_miss[i*4+1]));
        //         self.shapes.push(Shape::Point(point));
        //     }
        // }
        let mut basis1 = TraceTexelBasis::new(&self.texel, hit_miss);
        //console_log!("basis1.pair_texels {}", basis1.pair_texels.len());
        //console_log!("basis1.pair_texels {:?}", basis1.pair_texels);
        let (_, pair_buf_size1) = self.gpu.texture.make_row_rg32i(1, &mut basis1.pair_texels)?;
        let point_buf_size1 = ivec2(pair_buf_size1.x*3, pair_buf_size1.y*2);
        let trace_length = 300;
        let trace_buf_size = ivec2(pair_buf_size1.x, trace_length);
        //console_log!("trace_buf_size {:?}", trace_buf_size);
        self.trace_buffer = Some(TraceBuffer{
            point:   self.gpu.framebuffer.make_empty_rgba32f(2, point_buf_size1)?,
            segment: self.gpu.framebuffer.make_row_rgba32f(3, &mut basis1.uv_box_texels)?,
            honed:   self.gpu.framebuffer.make_multi_empty_rgba32f(5, pair_buf_size1, 3)?,
            trace:   self.gpu.framebuffer.make_multi_empty_rgba32f(8, trace_buf_size, 2)?,
        });
        self.trace(trace_length);
        let buff1 = &self.trace_buffer.as_ref().unwrap();
        let traces  = self.gpu.read(&buff1.trace, 0);
        let boxes   = self.gpu.read(&buff1.honed, 1);
        //console_log!("boxes {:?}", boxes);
        let centers = self.gpu.read(&buff1.trace, 1);
        let traced_curves = get_traced_curves(basis1.index_pairs, trace_buf_size, traces, boxes, centers);
        for TracedCurve{index_pair, curve0, curve1, center} in traced_curves {
            let IndexPair{g0, g1, f0, f1} = index_pair;
            self.facet_hits[g0][f0].push(curve0);
            self.facet_hits[g1][f1].push(curve1);
            self.shapes.push(Shape::Curve(center));
        }   
        Ok(())     
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