use glam::*;
use web_sys::WebGlProgram;
use crate::gpu::framebuffer::Framebuffer;
use crate::{rank0, Shape};
use crate::{gpu::GPU, Spatial3, DUP_0_TOL};
use super::{Hit, HitPair, HoneBuffer, Out, OutPair, TestPair};
use super::shaders2::{HIT_MISS_SOURCE, HONE_SOURCE, INIT_PALETTE_SOURCE};

// pub trait HitTest2 {
//     fn hit2(self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<MissPair>);
// }

// impl HitTest2 for Vec<Shape> {
    pub fn hit2(shapes: Vec<Shape>, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>) {
        let gpu = GPU::new().unwrap();
        let mut basis = HoneBasis2::new(&shapes, &pairs);
        gpu.texture.make_r32f(0, &mut basis.shape_texels).unwrap();
        let (_, pair_buf_size) = gpu.texture.make_rg32i(1, &mut basis.pair_texels).unwrap();
        let palette_buf_size = ivec2(pair_buf_size.x*3, pair_buf_size.y*2);
        let buffer = HoneBuffer{
            io:       gpu.framebuffer.make_rgba32f_with_empties(2, &mut basis.knot_texels, 2).unwrap(),
            palette0: gpu.framebuffer.make_multi_empty_rgba32f(4, palette_buf_size, 2).unwrap(),
            palette1: gpu.framebuffer.make_multi_empty_rgba32f(6, palette_buf_size, 2).unwrap(),
        };
        HitBasis2 {
            //curves:self, 
            //pairs: pairs.clone(), 
            basis, 
            buffer, 
            init_palette:     gpu.get_quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
            hone_palette:     gpu.get_quad_program_from_source(HONE_SOURCE).unwrap(),
            hit_miss_program: gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            gpu,
            spatial: Spatial3::new(0.1), 
            points:  vec![],
        }.make() // .expect("HitBasis2 should work for Vec<CurveShape>.hit()")
    }
//}

pub struct HitBasis2 {
    //curves: Vec<CurveShape>,
    //pairs:  Vec<TestPair>,
    basis:  HoneBasis2,
    buffer: HoneBuffer,
    init_palette: WebGlProgram,
    hone_palette: WebGlProgram,
    hit_miss_program: WebGlProgram,
    gpu: GPU,
    pub spatial:      Spatial3,
    pub points:       Vec<Vec3>,
}

impl HitBasis2 { 
    pub fn make(&mut self) -> (Vec<HitPair>, Vec<OutPair>) { 
        self.hone();
        let score = self.gpu.read(&self.buffer.io, 0);
        //console_log!("hit_miss {:?}", hit_miss);
        let points   = self.gpu.read(&self.buffer.io, 1);
        let mut hits = vec![];
        let mut outs = vec![];
        //let mut to_prints: Vec<f32> = vec![];
        for (i, pair) in self.basis.pairs.iter().enumerate() {
            let j = i * 4;
            if score[j] > -0.5 { // it's a hit
                //to_prints.extend(&[999., hit_miss[j], hit_miss[j+1], hit_miss[j+2], hit_miss[j+3]]);
                //log("hit!");
                let point = vec3(points[j+0], points[j+1], points[j+2]);
                let mut duplicate = false;
                for i in self.spatial.get(&point) {
                    if self.points[i].distance(point) < DUP_0_TOL { 
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    self.spatial.insert(&point, self.points.len());
                    self.points.push(point);
                    hits.push(HitPair {
                        test: pair.clone(),
                        // u0:   hit_miss[j+0],
                        // u1:   hit_miss[j+1],
                        // dot0: hit_miss[j+2],
                        // dot1: hit_miss[j+3], 
                        hits: (Hit{
                            u:   score[j+0],
                            dot: score[j+2],
                            shape: None,
                            twin: vec![],
                        },
                        Hit{
                            u:   score[j+1],
                            dot: score[j+3],
                            shape: None,
                            twin: vec![],
                        }),
                        shape: rank0(point),
                    }); 
                }
            }else{
                // if hit_miss[i*4+1].is_nan() || hit_miss[i*4+2].is_nan() || hit_miss[i*4+3].is_nan() {
                //     log("nan hit_miss in union3!");
                //     continue;
                // }
                if score[i*4] < -5. {continue}
                outs.push(OutPair { 
                    test:     pair.clone(),
                    outs: (
                        Out{dot:score[j+1], distance:score[j+3]},
                        Out{dot:score[j+2], distance:score[j+3]}
                    ),
                    // dots:     (hit_miss[j+1], hit_miss[j+2]), 
                    // distance: hit_miss[j+3],
                });
            }
        }
        // console_log!("hits {:?}", to_prints);
        // let wow: Vec<f32> = vec![1.234567891234567891234; 3];
        // console_log!("wow {}", wow[0]);
        // console_log!("f32::DIGITS {}", f32::DIGITS);
        (hits, outs)
    }
    fn hone(&self) {
        self.draw_init_hone_palette();
        for _ in 0..8 {
            self.draw_hone_palette(&self.buffer.palette1, 4);
            self.draw_hone_palette(&self.buffer.palette0, 6);
        }
        self.draw_hit_miss();
    }
    fn draw_init_hone_palette(&self){
        self.gpu.gl.use_program(Some(&self.init_palette));
        self.gpu.set_uniform_1i(&self.init_palette, "pair_tex",  1);
        self.set_curve_uniforms(&self.init_palette);
        self.gpu.set_uniform_1i(&self.init_palette, "io_tex", 2);
        self.gpu.draw(&self.buffer.palette0);
    }
    fn draw_hone_palette(&self, buff: &Framebuffer, i: i32) {
        self.gpu.gl.use_program(Some(&self.hone_palette));
        self.gpu.set_uniform_1i(&self.hone_palette, "pair_tex", 1);
        self.set_curve_uniforms(&self.hone_palette);
        self.set_arrow_uniforms(&self.hone_palette, i);
        self.gpu.draw(buff);
    }
    fn draw_hit_miss(&self){
        self.gpu.gl.use_program(Some(&self.hit_miss_program));
        self.gpu.set_uniform_1i(&self.hit_miss_program, "pair_tex", 1);
        self.set_curve_uniforms(&self.hit_miss_program);
        self.set_arrow_uniforms(&self.hit_miss_program, 4);
        self.gpu.draw(&self.buffer.io);
    }
    fn set_curve_uniforms(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "geom_tex", 0);
        self.gpu.set_uniform_1i(program, "max_knot_count", self.basis.max_knot_count);
    }
    fn set_arrow_uniforms(&self, program: &WebGlProgram, i: i32) {
        self.gpu.set_uniform_1i(program, "point_tex", i);
        self.gpu.set_uniform_1i(program, "delta_tex", i + 1);
    }
}

#[derive(Clone, Debug)]
struct IndexedKnot {
    index: usize,
    knot:  f32
}


#[derive(Default)]
pub struct HoneBasis2{
    pub pairs: Vec<TestPair>,
    pub pair_texels:  Vec<i32>,
    pub shape_texels: Vec<f32>,
    pub knot_texels:  Vec<f32>,
    pub max_knot_count: i32,
}

impl HoneBasis2 {
    pub fn new(shapes: &Vec<Shape>, pairs: &Vec<TestPair>) -> Self{
        let mut max_knot_count = 0;
        let mut index_pairs:  Vec<TestPair> = vec![];
        let mut knot_groups:  Vec<Vec<IndexedKnot>> = vec![]; // vec![]; shapes.len()
        let mut shape_texels: Vec<f32> = vec![];
        let mut pair_texels:  Vec<i32> = vec![];
        let mut knot_texels:  Vec<f32> = vec![];
        for shape in shapes {
            let mut indexed_knots: Vec<IndexedKnot> = vec![];
            //if shape.rank == 1 {
                if shape.basis.knots.len() > max_knot_count { 
                    max_knot_count = shape.basis.knots.len(); 
                }
                let texel_index = shape_texels.len();
                shape_texels.extend([
                    9000000., // + ci as f32,
                    shape.controls.len() as f32,
                    shape.basis.order as f32,
                    shape.basis.min,
                    shape.basis.max,
                ]); 
                shape_texels.push(0.);
                for i in 1..shape.basis.knots.len() { // -1 {
                    if shape.basis.knots[i-1] < shape.basis.knots[i] || i == shape.basis.order-1 { 
                        indexed_knots.push(IndexedKnot{
                            index: texel_index, 
                            knot:  shape.basis.knots[i],
                        }); 
                    }
                    shape_texels.push(shape.basis.knots[i]);
                }  
                shape_texels.extend(&shape.basis.weights);
                for control in &shape.controls {
                    shape_texels.extend(control.point(&[]).to_array());
                }
            //}
            knot_groups.push(indexed_knots);
        }
        for pair in pairs {
            for IndexedKnot{index:t0, knot:u0} in &knot_groups[pair.i0]{
                for IndexedKnot{index:t1, knot:u1} in &knot_groups[pair.i1]{
                    index_pairs.push(pair.clone());
                    pair_texels.push(*t0 as i32);
                    pair_texels.push(*t1 as i32);
                    knot_texels.extend(&[*u0, *u1, 0., 0.]);
                }  
            }  
        }
        // console_log!("u_groups {:?}", u_groups);
        // console_log!("index_pairs {:?}", index_pairs);
        // console_log!("pair_texels {:?}", pair_texels);
        // console_log!("u_texels {:?}", u_texels);
        HoneBasis2 {
            pairs: index_pairs,
            pair_texels,
            shape_texels,
            knot_texels,
            max_knot_count: max_knot_count as i32,
        }
    }
}