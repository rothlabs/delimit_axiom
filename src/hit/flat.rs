use glam::*;
use web_sys::WebGlProgram;
use crate::gpu::framebuffer::Framebuffer;
use crate::shape::*; 
use crate::{gpu::GPU, Spatial3, DUP_0_TOL};
use super::{Hit, HitPair, HoneBuffer, Out, OutPair, TestPair};
use super::spread::{spreads, Spread};
use super::shaders2::{HIT_MISS_SOURCE, HONE_SOURCE, INIT_PALETTE_SOURCE};

pub trait HitTest {
    fn hit(&self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>);
}

impl HitTest for Vec<Shape> {
    fn hit(&self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>) {
        let gpu = GPU::new().unwrap();
        let (indices, mut shapes) = self.texels();
        gpu.texture.make_r32f(0, &mut shapes).unwrap();
        let mut spreads = spreads(self, &pairs, &indices);

        
        let (_, index_size) = gpu.texture.make_rg32i(1, &mut spreads[1][1].index).unwrap();
        let palette_size = ivec2(index_size.x*3, index_size.y*2);
        let buffer = HoneBuffer {
            io:       gpu.framebuffer.make_rgba32f_with_empties(2, &mut spreads[1][1].param, 2).unwrap(),
            palette0: gpu.framebuffer.make_multi_empty_rgba32f(4, palette_size, 2).unwrap(),
            palette1: gpu.framebuffer.make_multi_empty_rgba32f(6, palette_size, 2).unwrap(),
        };
        //console_log!("shapes.max_knot_len() {}", shapes.max_knot_len());
        HitBasis2 {
            pairs: pairs.clone(), 
            spreads, 
            buffer, 
            init_palette:     gpu.get_quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
            hone_palette:     gpu.get_quad_program_from_source(HONE_SOURCE).unwrap(),
            hit_miss_program: gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            max_knot_len:     self.max_knot_len() as i32,
            gpu,
            spatial: Spatial3::new(0.1), 
            points:  vec![],
        }.make() // .expect("HitBasis2 should work for Vec<CurveShape>.hit()")
    }
}

pub struct HitBasis2 {
    //curves: Vec<CurveShape>,
    pairs:  Vec<TestPair>,
    spreads:  [Vec<Spread>; 3], // HoningTexels,
    buffer: HoneBuffer,
    init_palette: WebGlProgram,
    hone_palette: WebGlProgram,
    hit_miss_program: WebGlProgram,
    max_knot_len: i32,
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
        for (i, k) in self.spreads[1][1].pairs.iter().enumerate() {
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
                        test: self.pairs[*k].clone(),
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
                    test:     self.pairs[*k].clone(),
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
    fn hone(&self) { // TODO: add an index to call this for multiple buffers for different shape rank combos!
        self.draw_init_hone_palette();
        for _ in 0..8 {
            self.draw_hone_palette(&self.buffer.palette1, 4);
            self.draw_hone_palette(&self.buffer.palette0, 6);
        }
        self.draw_hit_miss();
    }
    fn draw_init_hone_palette(&self){
        self.gpu.gl.use_program(Some(&self.init_palette));
        self.gpu.set_uniform_1i(&self.init_palette, "index_texture",  1);
        self.set_curve_uniforms(&self.init_palette);
        self.gpu.set_uniform_1i(&self.init_palette, "io_tex", 2);
        self.gpu.draw(&self.buffer.palette0);
    }
    fn draw_hone_palette(&self, buff: &Framebuffer, i: i32) {
        self.gpu.gl.use_program(Some(&self.hone_palette));
        self.gpu.set_uniform_1i(&self.hone_palette, "index_texture", 1);
        self.set_curve_uniforms(&self.hone_palette);
        self.set_arrow_uniforms(&self.hone_palette, i);
        self.gpu.draw(buff);
    }
    fn draw_hit_miss(&self){
        self.gpu.gl.use_program(Some(&self.hit_miss_program));
        self.gpu.set_uniform_1i(&self.hit_miss_program, "index_texture", 1);
        self.set_curve_uniforms(&self.hit_miss_program);
        self.set_arrow_uniforms(&self.hit_miss_program, 4);
        self.gpu.draw(&self.buffer.io);
    }
    fn set_curve_uniforms(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "shape_texture", 0);
        self.gpu.set_uniform_1i(program, "max_knot_count", self.max_knot_len);
    }
    fn set_arrow_uniforms(&self, program: &WebGlProgram, i: i32) {
        self.gpu.set_uniform_1i(program, "point_tex", i);
        self.gpu.set_uniform_1i(program, "delta_tex", i + 1);
    }
}

// #[derive(Clone, Debug)]
// struct IndexedKnot {
//     index: usize,
//     knot:  f32
// }



// for pair in pairs {
//     for IndexedKnot{index:t0, knot:u0} in &knot_groups[pair.i0] {
//         for IndexedKnot{index:t1, knot:u1} in &knot_groups[pair.i1] {
//             index_pairs.push(pair.clone());
//             pair_texels.push(*t0 as i32);
//             pair_texels.push(*t1 as i32);
//             knot_texels.extend(&[*u0, *u1, 0., 0.]);
//         }  
//     }  
// }


// texels.push(0.); // first knot
//     for i in 1..shape.basis.knots.len() {
//         if shape.basis.knots[i-1] < shape.basis.knots[i] || i == shape.basis.order - 1 { // TODO: manually add IndexedKnot at shape.basis.order-1
//             indexed_knots.push(IndexedKnot{
//                 index: shape_index, 
//                 knot:  shape.basis.knots[i],
//             }); 
//         }
//         texels.push(shape.basis.knots[i]);
//     } 