use glam::*;
use crate::shape::*; 
use crate::{Spatial3, DUP_0_TOL};
use super::hone::Hone;
use super::{curve_hit, HitPair, Out, OutPair, TestPair};
use super::hone::spread::{ToSpread, Spread};
use crate::{get_state, set_state};

pub trait HitTest {
    fn hit(&self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>);
}

impl HitTest for Vec<Shape> {
    fn hit(&self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>) {
        let mut state = get_state();
        //let gpu = state.gpu; 
        let (indices, mut shapes) = self.texels();
        let mut spreads = self.spreads(&pairs, &indices);
        state.gpu.texture.r32f(0, &mut shapes).unwrap();
        //for spread in spreads[1] { // rank 1
        //}
        let mut spread = &mut spreads[1][1];
        state.hit.hone.buffer(&mut spread);
        state.hit.hone.max_knot_len = self.max_knot_len() as i32;
        state.hit.hone.draw();
        let score = score(&state.hit.hone, &spread, &pairs);
        //state.gpu = hone.gpu;
        set_state(state);
        score
    }
}



pub fn score(hone: &Hone, spread: &Spread, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>) { 
    let mut spatial = Spatial3::new(0.1); 
    let mut used_points: Vec<Vec3> = vec![];
    // let score  = hone.gpu.read(&hone.buffer.io, 0);
    // let points = one.gpu.read(&hone.buffer.io, 1);
    let (score, points) = hone.read();
    let mut hits = vec![];
    let mut outs = vec![];
    for (i, k) in spread.pairs.iter().enumerate() {
        let j = i * 4;
        if score[j] > -0.5 { // it's a hit
            let point = vec3(points[j+0], points[j+1], points[j+2]);
            let mut duplicate = false;
            for i in spatial.get(&point) {
                if used_points[i].distance(point) < DUP_0_TOL { 
                    duplicate = true;
                    break;
                }
            }
            if !duplicate {
                spatial.insert(&point, used_points.len());
                used_points.push(point);
                hits.push(HitPair {
                    test: pairs[*k].clone(), 
                    hits: (
                        curve_hit(score[j+0], score[j+2]), 
                        curve_hit(score[j+1], score[j+3]), 
                    ),
                    shape: rank0(point),
                }); 
            }
        }else{
            if score[i*4] < -5. {continue}
            outs.push(OutPair { 
                test:     pairs[*k].clone(),
                outs: (
                    Out{dot:score[j+1], distance:score[j+3]},
                    Out{dot:score[j+2], distance:score[j+3]}
                ),
            });
        }
    }
    (hits, outs)
}


        //let buffer = state.gpu.honing_buffer(&mut spread);
        //let program = state.hone.program[1][1];
        // let hone = Hone {
        //     // initial_program: gpu.quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
        //     // palette_program: gpu.quad_program_from_source(HONE_SOURCE).unwrap(),
        //     // score_program:   gpu.quad_program_from_source(SCORE_SOURCE).unwrap(),
        //     program,//: state.hone.program[1][1],
        //     max_knot_len:    self.max_knot_len() as i32, // TODO: max_knot_len should be unique for each honing op!!
        //     buffer, 
        //     gpu,
        // }.now(); 


// pub struct Hone {
//     // initial_program: WebGlProgram,
//     // palette_program: WebGlProgram,
//     // score_program:   WebGlProgram,
//     program: Program,
//     max_knot_len: i32,
//     buffer: HoningBuffer,
//     gpu: GPU,
// }

// impl Hone { 
//     fn now(self) -> Self { 
//         self.draw_initial();
//         for _ in 0..8 {
//             self.draw_palette(&self.buffer.palette1, 4);
//             self.draw_palette(&self.buffer.palette0, 6);
//         }
//         self.draw_score();
//         self
//     }
//     fn draw_initial(&self){
//         self.gpu.gl.use_program(Some(&self.program.initial));
//         self.gpu.set_uniform_1i(&self.program.initial, "index_texture",  1);
//         self.set_shape_uniforms(&self.program.initial);
//         self.gpu.set_uniform_1i(&self.program.initial, "io_tex", 2);
//         self.gpu.draw(&self.buffer.palette0);
//     }
//     fn draw_palette(&self, buff: &Framebuffer, i: i32) {
//         self.gpu.gl.use_program(Some(&self.program.palette));
//         self.gpu.set_uniform_1i(&self.program.palette, "index_texture", 1);
//         self.set_shape_uniforms(&self.program.palette);
//         self.set_arrow_uniforms(&self.program.palette, i);
//         self.gpu.draw(buff);
//     }
//     fn draw_score(&self){
//         self.gpu.gl.use_program(Some(&self.program.score));
//         self.gpu.set_uniform_1i(&self.program.score, "index_texture", 1);
//         self.set_shape_uniforms(&self.program.score);
//         self.set_arrow_uniforms(&self.program.score, 4);
//         self.gpu.draw(&self.buffer.io);
//     }
//     fn set_shape_uniforms(&self, program: &WebGlProgram) {
//         self.gpu.set_uniform_1i(program, "shape_texture", 0);
//         self.gpu.set_uniform_1i(program, "max_knot_count", self.max_knot_len);
//     }
//     fn set_arrow_uniforms(&self, program: &WebGlProgram, i: i32) {
//         self.gpu.set_uniform_1i(program, "point_tex", i);
//         self.gpu.set_uniform_1i(program, "delta_tex", i + 1);
//     }
// }


// if hit_miss[i*4+1].is_nan() || hit_miss[i*4+2].is_nan() || hit_miss[i*4+3].is_nan() {
            //     log("nan hit_miss in union3!");
            //     continue;
            // }


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