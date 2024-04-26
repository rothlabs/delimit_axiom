use std::f32::EPSILON;
use glam::*;
use web_sys::WebGlProgram;
use crate::gpu::framebuffer::Framebuffer;
use crate::{gpu::GPU, log, CurveShape, Spatial3, AT_0_TOL, AT_1_TOL, DOT_1_TOL, DUP_TOL, HIT_TOL, UV_MISS_BUMP};
use super::{HitPair2, HoneBuffer, MissPair, TestPair};
use super::shaders2::{HIT_MISS_SOURCE, HONE_SOURCE, INIT_PALETTE_SOURCE};

pub trait HitTest2 {
    fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair2>, Vec<MissPair>);
}

impl HitTest2 for Vec<CurveShape> {
    fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair2>, Vec<MissPair>) {
        let gpu = GPU::new().unwrap();
        let mut basis = HoneBasis2::new(&self, &pairs);
        gpu.texture.make_r32f(0, &mut basis.curve_texels).unwrap();
        let (_, pair_buf_size) = gpu.texture.make_rg32i(1, &mut basis.pair_texels).unwrap();
        let palette_buf_size = ivec2(pair_buf_size.x*3, pair_buf_size.y*2);
        let buffer = HoneBuffer{
            io:       gpu.framebuffer.make_rgba32f_with_empties(2, &mut basis.u_texels, 2).unwrap(),
            palette0: gpu.framebuffer.make_multi_empty_rgba32f(4, palette_buf_size, 2).unwrap(),
            palette1: gpu.framebuffer.make_multi_empty_rgba32f(6, palette_buf_size, 2).unwrap(),
        };
        HitBasis2 {
            //curves:self, 
            pairs: pairs.clone(), 
            basis, 
            buffer, 
            init_palette:     gpu.get_quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
            hone_palette:     gpu.get_quad_program_from_source(HONE_SOURCE).unwrap(),
            hit_miss_program: gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            gpu,
            spatial: Spatial3::new(), 
            points:  vec![],
        }.make() // .expect("HitBasis2 should work for Vec<CurveShape>.hit()")
    }
}

pub struct HitBasis2 {
    //curves: Vec<CurveShape>,
    pairs:  Vec<TestPair>,
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
    pub fn make(&mut self) -> (Vec<HitPair2>, Vec<MissPair>) { 
        self.hone();
        let hit_miss = self.gpu.read(&self.buffer.io, 0);
        let points   = self.gpu.read(&self.buffer.io, 1);
        let mut hits = vec![];
        let mut misses = vec![];
        for (i, pair) in self.pairs.iter().enumerate() {
            let j = i * 4;
            if hit_miss[j] > -0.5 { // it's a hit
                let point = vec3(points[j+0], points[j+1], points[j+2]);
                let mut duplicate = false;
                for i in self.spatial.get(&point) {
                    if self.points[i].distance(point) < DUP_TOL {
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    self.spatial.insert(&point, self.points.len());
                    self.points.push(point);
                    hits.push(HitPair2{
                        pair:     pair.clone(),
                        u0:   hit_miss[j+0],
                        u1:   hit_miss[j+1],
                        dot0: hit_miss[j+2],
                        dot1: hit_miss[j+3], 
                        point,
                    }); 
                }
            }else{
                // if hit_miss[i*4+1].is_nan() || hit_miss[i*4+2].is_nan() || hit_miss[i*4+3].is_nan() {
                //     log("nan hit_miss in union3!");
                //     continue;
                // }
                if hit_miss[i*4] < -5. {continue}
                misses.push(MissPair { 
                    pair:     pair.clone(),
                    dot0:     hit_miss[j+1], 
                    dot1:     hit_miss[j+2], 
                    distance: hit_miss[j+3],
                });
            }
        }
        (hits, misses)
    }
    fn hone(&self) {
        self.draw_init_hone_palette();
        for _ in 0..5 {
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


struct IndexedU {
    texel_index: usize,
    u: f32
}


#[derive(Default)]
pub struct HoneBasis2{
    pub index_pairs: Vec<TestPair>,
    pub pair_texels: Vec<i32>,
    pub curve_texels: Vec<f32>,
    pub u_texels: Vec<f32>,
    pub max_knot_count: i32,
}

impl HoneBasis2 {
    pub fn new(curves: &Vec<CurveShape>, pairs: &Vec<TestPair>) -> Self{
        let mut max_knot_count = 0;
        let mut index_pairs: Vec<TestPair> = vec![];
        let mut u_groups: Vec<Vec<IndexedU>> = vec![];
        let mut curve_texels: Vec<f32> = vec![];
        let mut pair_texels: Vec<i32> = vec![];
        let mut u_texels: Vec<f32> = vec![];
        for (ci, curve) in curves.iter().enumerate() {
            let mut u_indexes: Vec<IndexedU> = vec![];
            if curve.nurbs.knots.len() > max_knot_count { 
                max_knot_count = curve.nurbs.knots.len(); 
            }
            let texel_index = curve_texels.len();
            curve_texels.extend([
                9000000. + ci as f32,
                curve.controls.len() as f32,
                curve.nurbs.order as f32,
                curve.min,
                curve.max,
            ]); 
            for i in 0..curve.nurbs.knots.len()-1 {
                if curve.nurbs.knots[i] < curve.nurbs.knots[i+1] || i == curve.nurbs.knots.len() - curve.nurbs.order {
                    u_indexes.push(IndexedU{
                        texel_index, 
                        u: curve.nurbs.knots[i],
                    }); 
                }
                curve_texels.push(curve.nurbs.knots[i]);
            }  
            curve_texels.push(curve.nurbs.knots[curve.nurbs.knots.len()-1]);
            curve_texels.extend(&curve.nurbs.weights);
            for point in &curve.controls {
                curve_texels.extend(point.to_array());
            }
            u_groups.push(u_indexes);
        }
        for pair in pairs {
            for IndexedU{texel_index:t0, u:u0} in &u_groups[pair.i0]{
                for IndexedU{texel_index:t1, u:u1} in &u_groups[pair.i1]{
                    index_pairs.push(pair.clone());
                    pair_texels.push(*t0 as i32);
                    pair_texels.push(*t1 as i32);
                    u_texels.extend(&[*u0, *u1, 0., 0.]);
                }  
            }  
        }
        HoneBasis2 {
            index_pairs,
            pair_texels,
            curve_texels,
            u_texels,
            max_knot_count: max_knot_count as i32,
        }
    }
}