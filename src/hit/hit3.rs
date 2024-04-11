use crate::{log, CurveShape, FacetShape, Shape};
use crate::gpu::{framebuffer::Framebuffer, shader::COPY_FRAGMENT_SOURCE, GPU};
use glam::*;
use web_sys::WebGlProgram;
use super::basis3::{HoneBasis, TraceBasis};
use super::shader::{HIT_MISS_SOURCE, INIT_HONE_QUAD_SOURCE, HONE_QUAD_SOURCE, HONE_TRACE_SOURCE, TRACE_SOURCE};
use super::traced::{get_traced_curves, TracedCurve};
use super::{IndexPair, Miss, MissPair};


struct HoneBuffer {
    uv:    Framebuffer,
    dual:  Framebuffer,
    quad0: Framebuffer,
    quad1: Framebuffer,
}

struct TraceBuffer {
    dual:   Framebuffer,
    quad:   Framebuffer,
    trace:  Framebuffer,
}

//#[derive(Clone)]
pub struct HitBasis3 {
    pub facet_groups: Vec<Vec<FacetShape>>,
    pub tolerance: f32,
    pub step:      f32,
    pub length:    usize,
    pub facet_hits: Vec<Vec<Vec<Vec<CurveShape>>>>, 
    pub facet_miss: Vec<Vec<Vec<Vec<Miss>>>>, 
    pub shapes: Vec<Shape>,
    hone_basis: HoneBasis,
    //init_hone_dual:   WebGlProgram,   
    init_hone_quad:   WebGlProgram,
    hone_quad:        WebGlProgram,
    hit_miss_program: WebGlProgram, 
    // copy_program:     WebGlProgram,
    // trace_program:    WebGlProgram, 
    // hone_trace_program: WebGlProgram, 
    hone_buffer:   Option<HoneBuffer>,
    trace_buffer:  Option<TraceBuffer>,
    gpu: GPU,
}

impl HitBasis3 { 
    pub fn new(facet_groups: Vec<Vec<FacetShape>>) -> Self {
        let mut facet_hits = vec![];
        let mut facet_miss = vec![];
        for (gi, facet_group) in facet_groups.iter().enumerate() {
            facet_hits.push(facet_group.iter().map(|_| vec![vec![]; facet_groups.len()-gi+1]).collect());
            facet_miss.push(facet_group.iter().map(|_| vec![vec![]; facet_groups.len()-gi+1]).collect());
        }
        let gpu = GPU::new().unwrap();
        HitBasis3 {
            facet_groups,
            tolerance: 0.05,
            step:      0.8,
            length:    300,
            facet_hits,
            facet_miss,
            shapes: vec![],
            hone_basis: HoneBasis::default(),
            //init_hone_dual:      gpu.get_quad_program_from_source(INIT_HONE_QUAD_SOURCE).unwrap(),
            init_hone_quad:      gpu.get_quad_program_from_source(INIT_HONE_QUAD_SOURCE).unwrap(),
            hone_quad:           gpu.get_quad_program_from_source(HONE_QUAD_SOURCE).unwrap(),
            hit_miss_program:    gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            // copy_program:        gpu.get_quad_program_from_source(COPY_FRAGMENT_SOURCE).unwrap(),
            // trace_program:       gpu.get_quad_program_from_source(TRACE_SOURCE).unwrap(),
            // hone_trace_program:  gpu.get_quad_program_from_source(HONE_TRACE_SOURCE).unwrap(),
            hone_buffer: None,
            trace_buffer: None,
            gpu,
        }
    }
    pub fn make(&mut self) -> Result<(), String> { 
        let mut hone_basis = HoneBasis::new(&self.facet_groups);
        self.gpu.texture.make_r32f(0, &mut hone_basis.facet_texels)?;
        let (_, pair_buf_size0) = self.gpu.texture.make_rg32i(1, &mut hone_basis.pair_texels)?;
        let dual_buf_size0 = ivec2(pair_buf_size0.x,   pair_buf_size0.y*2);
        let quad_buf_size0 = ivec2(pair_buf_size0.x*3, pair_buf_size0.y*2);
        self.hone_buffer = Some(HoneBuffer{
            uv:    self.gpu.framebuffer.make_rgba32f(2, &mut hone_basis.uv_texels)?,
            dual:  self.gpu.framebuffer.make_multi_empty_rgba32f(3, dual_buf_size0, 3)?,
            quad0: self.gpu.framebuffer.make_multi_empty_rgba32f(6, quad_buf_size0, 3)?,
            quad1: self.gpu.framebuffer.make_multi_empty_rgba32f(9, quad_buf_size0, 3)?,
        });
        self.hone_basis = hone_basis;
        self.hone();
        let buff0 = &self.hone_buffer.as_ref().unwrap();
            // let tex_data = self.gpu.read(&buff0.quad0, 0);
            // console_log!("tex_data len {}", tex_data.len());
            // console_log!("index_pairs len {}", self.hone_basis.index_pairs.len());
            // let len = tex_data.len() / 2;
            // for i in 0..self.hone_basis.index_pairs.len() {
            //     //let IndexPair{g0, g1, i0, i1} = self.hone_basis.index_pairs[i];
            //     //let point = self.facet_groups[g1][i1].get_point(vec2(hit_miss[i*4], hit_miss[i*4+1]));
            //     let point = vec3(tex_data[len+ i*4], tex_data[len+ i*4 +1], tex_data[len+ i*4 +2]);
            //     self.shapes.push(Shape::Point(point));
            // }
        let hit_miss = self.gpu.read(&buff0.uv, 0);
                console_log!("hit_miss len {}", hit_miss.len());
                console_log!("index_pairs len {}", self.hone_basis.index_pairs.len());
                for i in 0..self.hone_basis.index_pairs.len() {
                    if hit_miss[i*4] > -0.5 {
                        let IndexPair{g0, g1, i0, i1} = self.hone_basis.index_pairs[i];
                        let point = self.facet_groups[g0][i0].get_point(vec2(hit_miss[i*4], hit_miss[i*4+1]));
                        self.shapes.push(Shape::Point(point));
                    }
                }
        // let mut trace_basis = TraceBasis::new(&self.hone_basis, hit_miss);
        // let (_, pair_buf_size1) = self.gpu.texture.make_rg32i(1, &mut trace_basis.pair_texels)?;
        // let _ = self.gpu.texture.make_rgba32f(2, &mut trace_basis.uv_texels)?;
        // let _ = self.gpu.texture.make_rgba32f(3, &mut trace_basis.box_texels)?;
        // let dual_buf_size1 = ivec2(pair_buf_size1.x*2, pair_buf_size1.y*2);
        // let quad_buf_size1 = ivec2(pair_buf_size1.x*6, pair_buf_size1.y*2);
        // let trace_length = 300;
        // let trace_buf_size = ivec2(pair_buf_size1.x*2, trace_length);
        // self.trace_buffer = Some(TraceBuffer{
        //     dual:  self.gpu.framebuffer.make_multi_empty_rgba32f(4,  dual_buf_size1, 4)?, // point, deriv_u,u, deriv_v,v, box
        //     quad:  self.gpu.framebuffer.make_multi_empty_rgba32f(8,  quad_buf_size1, 4)?, // point, deriv_u,u, deriv_v,v, box
        //     trace: self.gpu.framebuffer.make_multi_empty_rgba32f(12, trace_buf_size, 4)?, // uvs, points, uv_dirs, dirs
        // });
        // self.trace(trace_length);
        // let buff1   = &self.trace_buffer.as_ref().unwrap();
        // let boxes   = self.gpu.read(&buff1.dual,  3);
        // let traces  = self.gpu.read(&buff1.trace, 0);
        // let centers = self.gpu.read(&buff1.trace, 1);
        // let uv_dirs = self.gpu.read(&buff1.trace, 2);
        // let dirs    = self.gpu.read(&buff1.trace, 3);
        // let traced_curves = get_traced_curves(trace_basis.index_pairs, trace_buf_size, traces, boxes, centers, uv_dirs, dirs);
        // for TracedCurve{index_pair, curve0, curve1, center} in traced_curves {
        //     let IndexPair{g0, g1, i0, i1} = index_pair;
        //     self.facet_hits[g0][i0][g1-g0-1].push(curve0);
        //     self.facet_hits[g1][i1][0].push(curve1);
        //     self.shapes.push(Shape::Curve(center));
        // }  
        // for MissPair{index, distance, dot0, dot1} in trace_basis.misses {
        //     let IndexPair{g0, g1, i0, i1} = index;
        //     self.facet_miss[g0][i0][g1-g0-1].push(Miss{distance, dot:dot0});
        //     self.facet_miss[g1][i1][0].push(Miss{distance, dot:dot1});
        // }  
        Ok(())     
    }

    fn set_facet_uniforms(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "facet_tex", 0);
        self.gpu.set_uniform_1i(program, "pair_tex",  1);
        self.gpu.set_uniform_1i(program, "max_facet_length", self.hone_basis.max_facet_length);
        self.gpu.set_uniform_1i(program, "max_knot_count",   self.hone_basis.max_knot_count);
    }

    fn set_ray_uniforms(&self, program: &WebGlProgram, i: i32) {
        self.gpu.set_uniform_1i(program, "point_tex",    i);
        self.gpu.set_uniform_1i(program, "deriv_tex_u",  i + 1);
        self.gpu.set_uniform_1i(program, "deriv_tex_v",  i + 2);
    }

    fn hone(&self) {
        let buff = &self.hone_buffer.as_ref().unwrap();
        //self.draw_init_hone_dual();
        self.draw_init_hone_quad();
        for _ in 0..3 {
            self.draw_hone_quad(&buff.quad1, 6);
            self.draw_hone_quad(&buff.quad0, 9);
        }
        self.draw_hit_miss();
    }

    // fn draw_init_hone_dual(&self){
    //     self.gpu.gl.use_program(Some(&self.init_hone_dual));
    //     self.set_facet_uniforms(&self.init_hone_dual);
    //     self.gpu.set_uniform_1i(&self.init_hone_dual, "uv_tex", 2);
    //     self.gpu.draw(&self.hone_buffer.as_ref().unwrap().dual);
    // }

    fn draw_init_hone_quad(&self){
        self.gpu.gl.use_program(Some(&self.init_hone_quad));
        self.set_facet_uniforms(&self.init_hone_quad);
        self.gpu.set_uniform_1i(&self.init_hone_quad, "uv_tex", 2);
        //self.set_ray_uniforms(&self.init_hone_quad, 3);
        self.gpu.draw(&self.hone_buffer.as_ref().unwrap().quad0);
    }

    fn draw_hone_quad(&self, buff: &Framebuffer, i: i32) {
        self.gpu.gl.use_program(Some(&self.hone_quad));
        self.set_facet_uniforms(&self.hone_quad);
        self.set_ray_uniforms(&self.hone_quad, i);
        self.gpu.draw(buff);
    }

    fn draw_hit_miss(&self){
        self.gpu.gl.use_program(Some(&self.hit_miss_program));
        self.set_facet_uniforms(&self.hit_miss_program);
        self.set_ray_uniforms(&self.hit_miss_program, 6);
        self.gpu.draw(&self.hone_buffer.as_ref().unwrap().uv);
    }

    // fn trace(&self, length: i32){
    //     self.draw_init_trace_dual();
    //     self.draw_trace_quad();
    //     self.draw_trace_segment(0);
    //     for y in 0..length {
    //         self.draw_trace_dual();
    //         self.draw_trace_quad();
    //         self.draw_trace_segment(y);
    //     }
    // }

    // fn draw_init_trace_dual(&self){
    //     self.gpu.gl.use_program(Some(&self.init_trace_dual));
    //     self.set_facet_uniforms(&self.init_trace_dual);
    //     self.gpu.set_uniform_1i(&self.init_trace_dual, "uv_tex", 2);
    //     self.gpu.set_uniform_1i(&self.init_trace_dual, "uv_tex", 3);
    //     self.gpu.draw(&self.hone_buffer.as_ref().unwrap().dual);
    // }

    // fn draw_points(&self, uv_i: i32) {
    //     self.gpu.gl.use_program(Some(&self.init_hone_dual));
    //     self.set_facet_uniforms(&self.init_hone_dual);
    //     self.gpu.set_uniform_1i(&self.init_hone_dual, "uv_tex",  uv_i);
    //     self.gpu.draw(&self.hone_buffer.as_ref().unwrap().dual)
    // }

    // fn draw_trace_points(&self, uv_i: i32) {
    //     self.gpu.gl.use_program(Some(&self.init_hone_dual));
    //     self.set_facet_uniforms(&self.init_hone_dual);
    //     self.gpu.set_uniform_1i(&self.init_hone_dual, "uv_tex",  uv_i);
    //     self.gpu.draw(&self.trace_buffer.as_ref().unwrap().point)
    // }

    // fn hone_trace(&self) {
    //     self.gpu.gl.use_program(Some(&self.hone_trace_program));
    //     self.set_facet_uniforms(&self.hone_trace_program);
    //     self.gpu.set_uniform_1i(&self.hone_trace_program, "point_tex", 2);
    //     self.gpu.set_uniform_1i(&self.hone_trace_program, "uv_tex",  3);
    //     self.gpu.set_uniform_1i(&self.hone_trace_program, "box_tex", 4);
    //     self.gpu.draw(&self.trace_buffer.as_ref().unwrap().honed);
    // }

    // fn copy_trace(&self, y: i32) {
    //     self.gpu.gl.use_program(Some(&self.copy_program));
    //     self.gpu.set_uniform_1i(&self.copy_program, "source_tex0",  7); // 5
    //     self.gpu.set_uniform_1i(&self.copy_program, "source_tex1",  9); // 7
    //     self.gpu.set_uniform_1i(&self.copy_program, "source_tex2",  5); 
    //     self.gpu.set_uniform_1i(&self.copy_program, "source_tex3",  6); 
    //     self.gpu.set_uniform_2i(&self.copy_program, "viewport_position", IVec2::Y*y);
    //     self.gpu.draw_at_pos(&self.trace_buffer.as_ref().unwrap().trace, IVec2::Y*y);
    // }

    // fn trace_segment(&self) {
    //     self.gpu.gl.use_program(Some(&self.trace_program));
    //     self.set_facet_uniforms(&self.trace_program);
    //     self.gpu.set_uniform_1i(&self.trace_program, "point_tex", 2);
    //     self.gpu.set_uniform_1i(&self.trace_program, "uv_tex",  7); // 5
    //     self.gpu.set_uniform_1i(&self.trace_program, "box_tex", 8); // 6
    //     self.gpu.draw(&self.trace_buffer.as_ref().unwrap().segment);
    // }
}