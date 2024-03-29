use crate::{CurveShape, FacetShape, Shape};
use crate::gpu::{framebuffer::Framebuffer, shader::COPY_FRAGMENT_SOURCE, GPU};
use glam::*;
use web_sys::WebGlProgram;
use super::basis3::{HoneBasis, TraceBasis};
use super::shader::{HIT_MISS_SOURCE, HONE_SOURCE, HONE_TRACE_SOURCE, POINT_SOURCE, TRACE_SOURCE};
use super::traced::{get_traced_curves, TracedCurve};
use super::{IndexPair, Miss, MissPair};


struct HoneBuffer {
    point: Framebuffer,
    uv0:   Framebuffer,
    uv1:   Framebuffer,
}

struct TraceBuffer {
    point:   Framebuffer,
    segment: Framebuffer,
    honed:   Framebuffer,
    trace:   Framebuffer,
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
    point_program:    WebGlProgram,   
    hone_program:     WebGlProgram,
    hit_miss_program: WebGlProgram, 
    copy_program:     WebGlProgram,
    trace_program:    WebGlProgram, 
    hone_trace_program: WebGlProgram, 
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
            point_program:      gpu.get_quad_program_from_source(POINT_SOURCE).unwrap(),
            hone_program:       gpu.get_quad_program_from_source(HONE_SOURCE).unwrap(),
            hit_miss_program:   gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            copy_program:       gpu.get_quad_program_from_source(COPY_FRAGMENT_SOURCE).unwrap(),
            trace_program:      gpu.get_quad_program_from_source(TRACE_SOURCE).unwrap(),
            hone_trace_program: gpu.get_quad_program_from_source(HONE_TRACE_SOURCE).unwrap(),
            hone_buffer: None,
            trace_buffer: None,
            gpu,
        }
    }
    pub fn make(&mut self) -> Result<(), String> { 
        let mut hone_basis = HoneBasis::new(&self.facet_groups);
        self.gpu.texture.make_r32f(0, &mut hone_basis.facet_texels)?;
        let (_, pair_buf_size0) = self.gpu.texture.make_rg32i(1, &mut hone_basis.pair_texels)?;
        let point_buf_size0 = ivec2(pair_buf_size0.x*3, pair_buf_size0.y*2);
        self.hone_buffer = Some(HoneBuffer{
            point: self.gpu.framebuffer.make_empty_rgba32f(2, point_buf_size0)?,
            uv0:   self.gpu.framebuffer.make_rgba32f(3, &mut hone_basis.uv_texels)?,
            uv1:   self.gpu.framebuffer.make_empty_rgba32f(4, pair_buf_size0)?,
        });
        self.hone_basis = hone_basis;
        self.hone_to_hit_or_miss();
        let buff0 = &self.hone_buffer.as_ref().unwrap();
        let hit_miss = self.gpu.read(&buff0.uv1, 0);
        // for i in 0..hit_miss.len()/4 {
        //     if hit_miss[i*4] > -0.5 {
        //         let IndexPair{g0, g1, f0, f1} = self.texel.index_pairs[i];
        //         let point = self.facet_groups[g0][f0].get_point_at_uv(vec2(hit_miss[i*4], hit_miss[i*4+1]));
        //         self.shapes.push(Shape::Point(point));
        //     }
        // }
        let mut trace_basis = TraceBasis::new(&self.hone_basis, hit_miss);
        let (_, pair_buf_size1) = self.gpu.texture.make_row_rg32i(1, &mut trace_basis.pair_texels)?;
        let point_buf_size1 = ivec2(pair_buf_size1.x*3, pair_buf_size1.y*2);
        let trace_length = 300;
        let trace_buf_size = ivec2(pair_buf_size1.x, trace_length);
        self.trace_buffer = Some(TraceBuffer{
            point:   self.gpu.framebuffer.make_empty_rgba32f(2, point_buf_size1)?,
            segment: self.gpu.framebuffer.make_row_rgba32f(3, &mut trace_basis.uv_box_texels)?,
            honed:   self.gpu.framebuffer.make_multi_empty_rgba32f(5, pair_buf_size1, 3)?,
            trace:   self.gpu.framebuffer.make_multi_empty_rgba32f(8, trace_buf_size, 2)?,
        });
        self.trace(trace_length);
        let buff1 = &self.trace_buffer.as_ref().unwrap();
        let traces  = self.gpu.read(&buff1.trace, 0);
        let boxes   = self.gpu.read(&buff1.honed, 1);
        let centers = self.gpu.read(&buff1.trace, 1);
        let traced_curves = get_traced_curves(trace_basis.index_pairs, trace_buf_size, traces, boxes, centers);
        for TracedCurve{index_pair, curve0, curve1, center} in traced_curves {
            let IndexPair{g0, g1, i0, i1} = index_pair;
            self.facet_hits[g0][i0][g1-g0-1].push(curve0);
            self.facet_hits[g1][i1][0].push(curve1);
            self.shapes.push(Shape::Curve(center));
        }  
        for MissPair{index, distance, dot0, dot1} in trace_basis.misses {
            let IndexPair{g0, g1, i0, i1} = index;
            self.facet_miss[g0][i0][g1-g0-1].push(Miss{distance, dot:dot0});
            self.facet_miss[g1][i1][0].push(Miss{distance, dot:dot1});
        }  
        Ok(())     
    }

    fn hone_to_hit_or_miss(&self) {
        let buff = &self.hone_buffer.as_ref().unwrap();
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

    fn trace(&self, length: i32){
        for y in 0..length {
            self.draw_trace_points(3);
            self.hone_trace();
            self.copy_trace(y);
            self.draw_trace_points(5);
            self.trace_segment();
        }
    }

    fn set_uniform_basis(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "max_facet_length", self.hone_basis.max_facet_length);
        self.gpu.set_uniform_1i(program, "max_knot_count",   self.hone_basis.max_knot_count);
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
        self.gpu.draw(&self.hone_buffer.as_ref().unwrap().point)
    }

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
}