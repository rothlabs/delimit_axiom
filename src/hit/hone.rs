pub mod program;
pub mod spread;

use glam::*;
use program::Program;
use web_sys::WebGlProgram;
use crate::gpu::{framebuffer::Framebuffer, GPU};
use self::{spread::Spread};

use super::HoningBuffer;

// pub struct HoneState {
//     pub program: Vec<Vec<Program>>,
// }

// pub fn init_hone(gpu: GPU) -> HoneState {
//     HoneState {
//         program: init_program(gpu),
//     }
// }

pub struct Hone {
    ranks: (usize, usize),
    programs: Vec<Vec<Program>>,
    pub max_knot_len: i32,
    buffer: HoningBuffer,
    gpu: GPU,
}

impl Hone { 
    pub fn new(gpu: &GPU) -> Self {
        Hone {
            ranks: (1, 1),
            programs: Program::new(gpu),
            max_knot_len: 0,
            buffer: HoningBuffer {
                io:       gpu.framebuffer.make_empty_rgba32f(2, ivec2(32, 32)).unwrap(),
                palette0: gpu.framebuffer.make_empty_rgba32f(3, ivec2(32, 32)).unwrap(),
                palette1: gpu.framebuffer.make_empty_rgba32f(4, ivec2(32, 32)).unwrap(),
            },
            gpu: gpu.clone(),
        }
    } 
    pub fn buffer(&mut self, spread: &mut Spread) -> &mut Self {
        let (_, index_size) = self.gpu.texture.rg32i(1, &mut spread.index).unwrap();
        let palette_size = ivec2(index_size.x*3, index_size.y*2);
        self.buffer = HoningBuffer {
            io:       self.gpu.framebuffer.make_rgba32f_with_empties(2, &mut spread.param, 2).unwrap(),
            palette0: self.gpu.framebuffer.make_multi_empty_rgba32f(4, palette_size, 2).unwrap(),
            palette1: self.gpu.framebuffer.make_multi_empty_rgba32f(6, palette_size, 2).unwrap(),
        };
        self
    }
    //pub fn max_knot_len(&mut )
    pub fn draw(&mut self) { 
        self.draw_initial();
        for _ in 0..8 {
            self.draw_palette(&self.buffer.palette1, 4);
            self.draw_palette(&self.buffer.palette0, 6);
        }
        self.draw_score();
    }
    fn draw_initial(&self){
        let program = &self.programs[self.ranks.0][self.ranks.1].initial;
        self.gpu.gl.use_program(Some(program));
        self.gpu.set_uniform_1i(program, "index_texture",  1);
        self.set_shape_uniforms(program);
        self.gpu.set_uniform_1i(program, "io_tex", 2);
        self.gpu.draw(&self.buffer.palette0);
    }
    fn draw_palette(&self, buff: &Framebuffer, i: i32) {
        let program = &self.programs[self.ranks.0][self.ranks.1].palette;
        self.gpu.gl.use_program(Some(program));
        self.gpu.set_uniform_1i(program, "index_texture", 1);
        self.set_shape_uniforms(program);
        self.set_arrow_uniforms(program, i);
        self.gpu.draw(buff);
    }
    fn draw_score(&self){
        let program = &self.programs[self.ranks.0][self.ranks.1].score;
        self.gpu.gl.use_program(Some(program));
        self.gpu.set_uniform_1i(program, "index_texture", 1);
        self.set_shape_uniforms(program);
        self.set_arrow_uniforms(program, 4);
        self.gpu.draw(&self.buffer.io);
    }
    fn set_shape_uniforms(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "shape_texture", 0);
        self.gpu.set_uniform_1i(program, "max_knot_count", self.max_knot_len);
    }
    fn set_arrow_uniforms(&self, program: &WebGlProgram, i: i32) {
        self.gpu.set_uniform_1i(program, "point_tex", i);
        self.gpu.set_uniform_1i(program, "delta_tex", i + 1);
    }
    pub fn read(&self) -> (Vec<f32>, Vec<f32>) {
        (
            self.gpu.read(&self.buffer.io, 0),
            self.gpu.read(&self.buffer.io, 1),
        )
    }
}