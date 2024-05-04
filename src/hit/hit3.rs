use crate::{log, Shape};
use crate::gpu::{framebuffer::Framebuffer, GPU};
use glam::*;
use web_sys::WebGlProgram;
use super::basis3::{HoneBasis, TraceBasis};
use super::traced::{get_traced_curves, TracedCurve};
use super::{TestPair, HitPair3, MissPair};
use super::shaders3::{
    INIT_HONE_PALETTE_SOURCE, HONE_PALETTE_SOURCE, HIT_MISS_SOURCE, 
    INIT_TRACE_PALETTE_SOURCE, TRACE_SEGMENT_SOURCE, TRACE_DUAL_SOURCE, TRACE_PALETTE_SOURCE, BOXES_DUAL,
};

pub trait HitTest3 {
    fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair3>, Vec<MissPair>);
}

impl HitTest3 for Vec<Shape> {
    fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair3>, Vec<MissPair>) {
        let gpu = GPU::new().unwrap();
        HitBasis3 {
            facets: self,
            pairs: pairs.clone(),
            shapes: vec![],
            hone_basis: HoneBasis::default(),
            trace_count: 0,
            init_hone_palette:  gpu.get_quad_program_from_source(INIT_HONE_PALETTE_SOURCE).unwrap(),
            hone_palette:       gpu.get_quad_program_from_source(HONE_PALETTE_SOURCE).unwrap(),
            hit_miss_program:   gpu.get_quad_program_from_source(HIT_MISS_SOURCE).unwrap(),
            init_trace_palette: gpu.get_quad_program_from_source(INIT_TRACE_PALETTE_SOURCE).unwrap(),
            trace_segment:      gpu.get_quad_program_from_source(TRACE_SEGMENT_SOURCE).unwrap(),
            trace_dual:         gpu.get_quad_program_from_source(TRACE_DUAL_SOURCE).unwrap(),
            trace_palette:      gpu.get_quad_program_from_source(TRACE_PALETTE_SOURCE).unwrap(),
            boxes_dual:         gpu.get_quad_program_from_source(BOXES_DUAL).unwrap(),
            hone_buffer: None,
            trace_buffer: None,
            gpu,
        }.make().expect("HitBasis3 should work for Vec<CurveShape>.hit()")
    }
}



struct HoneBuffer {
    uv:    Framebuffer,
    palette0: Framebuffer,
    palette1: Framebuffer,
}

struct TraceBuffer {
    dual:    Framebuffer,
    palette: Framebuffer,
    trace:   Framebuffer,
    boxes:   Framebuffer,
}

//#[derive(Clone)]
pub struct HitBasis3 {
    pub facets: Vec<Shape>,
    pub pairs: Vec<TestPair>,
    //pub facet_hits: Vec<Hit3>, // Vec<TracedCurve>, // 
    //pub facet_miss: Vec<Miss3>, // Vec<Miss>, // 
    pub shapes: Vec<Shape>,
    hone_basis: HoneBasis,
    trace_count: i32,
    init_hone_palette:  WebGlProgram,
    hone_palette:       WebGlProgram,
    hit_miss_program:   WebGlProgram, 
    init_trace_palette: WebGlProgram,
    trace_segment:      WebGlProgram,
    trace_dual:         WebGlProgram,
    trace_palette:      WebGlProgram, 
    boxes_dual:         WebGlProgram, 
    hone_buffer:   Option<HoneBuffer>,
    trace_buffer:  Option<TraceBuffer>,
    gpu: GPU,
}

impl HitBasis3 { 
    pub fn make(&mut self) -> Result<(Vec<HitPair3>, Vec<MissPair>), String> { 
        let mut hone_basis = HoneBasis::new(&self.facets, &self.pairs);
        self.gpu.texture.make_r32f(0, &mut hone_basis.facet_texels)?;
        let (_, pair_buf_size) = self.gpu.texture.make_rg32i(1, &mut hone_basis.pair_texels)?;
        let palette_buf_size = ivec2(pair_buf_size.x*3, pair_buf_size.y*2);
        self.hone_buffer = Some(HoneBuffer{
            uv:       self.gpu.framebuffer.make_rgba32f(2, &mut hone_basis.uv_texels)?,
            palette0: self.gpu.framebuffer.make_multi_empty_rgba32f(3, palette_buf_size, 3)?,
            palette1: self.gpu.framebuffer.make_multi_empty_rgba32f(6, palette_buf_size, 3)?,
        });
        self.hone_basis = hone_basis;
        self.hone();
        let buff0 = &self.hone_buffer.as_ref().unwrap();
            // let tex_data = self.gpu.read(&buff0.palette0, 0);
            // console_log!("tex_data len {}", tex_data.len());
            // console_log!("index_pairs len {}", self.hone_basis.index_pairs.len());
            // let len = tex_data.len() / 2;
            // for i in 0..self.hone_basis.index_pairs.len() {
            //     //let IndexPair{g0, g1, i0, i1} = self.hone_basis.index_pairs[i];
            //     //let point = self.facets[g1][i1].get_point(vec2(hit_miss[i*4], hit_miss[i*4+1]));
            //     let point = vec3(tex_data[len+ i*4], tex_data[len+ i*4 +1], tex_data[len+ i*4 +2]);
            //     self.shapes.push(Shape::Point(point));
            // }
        let hit_miss = self.gpu.read(&buff0.uv, 0);
                // //console_log!("hit_miss len {}", hit_miss.len());
                // //console_log!("index_pairs len {}", self.hone_basis.index_pairs.len());
                // for i in 0..self.hone_basis.index_pairs.len() {
                //     if hit_miss[i*4] > -0.5 {
                //         let TestPair{i0, i1, reverse} = self.hone_basis.index_pairs[i];
                //         let point = self.facets[i0].get_point(vec2(hit_miss[i*4], hit_miss[i*4+1]));
                //         self.shapes.push(Shape::Point(point));
                //         let point = self.facets[i1].get_point(vec2(hit_miss[i*4+2], hit_miss[i*4+3]));
                //         self.shapes.push(Shape::Point(point));
                //     }
                // }
        let mut trace_basis = TraceBasis::new(&self.hone_basis, hit_miss);
        let (_, pair_buf_size) = self.gpu.texture.make_rg32i(1, &mut trace_basis.pair_texels)?;
        let _ = self.gpu.texture.make_rgba32f(2, &mut trace_basis.uv_texels)?;
               // let _ = self.gpu.texture.make_rgba32f(3, &mut trace_basis.box_texels)?;
        let dual_buf_size    = ivec2(pair_buf_size.x,   pair_buf_size.y*2);
        let palette_buf_size = ivec2(pair_buf_size.x*3, pair_buf_size.y*2);
        let trace_length = 250;
        //console_log!("trace_basis.index_pairs.len() {}", trace_basis.index_pairs.len());
        //console_log!("trace_basis.pair_texels.len() {}", trace_basis.pair_texels.len());
        self.trace_count = trace_basis.index_pairs.len() as i32;
        //console_log!("trace_count {}", self.trace_count);
        let trace_buf_size = ivec2(self.trace_count * 2, trace_length);
        let boxes_buf_size = ivec2(self.trace_count * 2, 1);
        self.trace_buffer = Some(TraceBuffer{
            boxes:   self.gpu.framebuffer.make_empty_rgba32f(3, boxes_buf_size)?,
            dual:    self.gpu.framebuffer.make_multi_empty_rgba32f(4,  dual_buf_size,    4)?, // point, deriv_u,u, deriv_v,v, box
            palette: self.gpu.framebuffer.make_multi_empty_rgba32f(8,  palette_buf_size, 4)?, // point, deriv_u,u, deriv_v,v, box
            trace:   self.gpu.framebuffer.make_multi_empty_rgba32f(12, trace_buf_size,   4)?, // origins, vectors, uvs, uv_vectors
        });
        self.trace(trace_length);
        let buff1      = &self.trace_buffer.as_ref().unwrap();
        let boxes      = self.gpu.read(&buff1.boxes, 0);
        let origins    = self.gpu.read(&buff1.trace, 0);
        let vectors    = self.gpu.read(&buff1.trace, 1);
        let uvs        = self.gpu.read(&buff1.trace, 2);
        let uv_vectors = self.gpu.read(&buff1.trace, 3);
        let hits = get_traced_curves(trace_basis.index_pairs, trace_buf_size, uvs, boxes, origins, uv_vectors, vectors);
        //self.facet_miss = trace_basis.misses;  
        Ok((hits, trace_basis.misses))     
    }

    fn set_facet_uniforms(&self, program: &WebGlProgram) {
        self.gpu.set_uniform_1i(program, "geom_tex", 0);
        self.gpu.set_uniform_1i(program, "max_facet_length", self.hone_basis.max_facet_length);
        self.gpu.set_uniform_1i(program, "max_knot_count",   self.hone_basis.max_knot_count);
    }

    fn set_arrow_uniforms(&self, program: &WebGlProgram, i: i32) {
        self.gpu.set_uniform_1i(program, "point_tex",    i);
        self.gpu.set_uniform_1i(program, "delta_tex_u", i + 1);
        self.gpu.set_uniform_1i(program, "delta_tex_v", i + 2);
    }

    fn hone(&self) {
        let buff = &self.hone_buffer.as_ref().unwrap();
        self.draw_init_hone_palette();
        for _ in 0..5 {
            self.draw_hone_palette(&buff.palette1, 3);
            self.draw_hone_palette(&buff.palette0, 6);
        }
        self.draw_hit_miss();
    }

    fn draw_init_hone_palette(&self){
        self.gpu.gl.use_program(Some(&self.init_hone_palette));
        self.gpu.set_uniform_1i(&self.init_hone_palette, "pair_tex",  1);
        self.set_facet_uniforms(&self.init_hone_palette);
        self.gpu.set_uniform_1i(&self.init_hone_palette, "io_tex", 2);
        self.gpu.draw(&self.hone_buffer.as_ref().unwrap().palette0);
    }

    fn draw_hone_palette(&self, buff: &Framebuffer, i: i32) {
        self.gpu.gl.use_program(Some(&self.hone_palette));
        self.gpu.set_uniform_1i(&self.hone_palette, "pair_tex", 1);
        self.set_facet_uniforms(&self.hone_palette);
        self.set_arrow_uniforms(&self.hone_palette, i);
        self.gpu.draw(buff);
    }

    fn draw_hit_miss(&self){
        self.gpu.gl.use_program(Some(&self.hit_miss_program));
        self.gpu.set_uniform_1i(&self.hit_miss_program, "pair_tex", 1);
        self.set_facet_uniforms(&self.hit_miss_program);
        self.set_arrow_uniforms(&self.hit_miss_program, 3);
        self.gpu.draw(&self.hone_buffer.as_ref().unwrap().uv);
    }

    fn trace(&self, length: i32){
        self.draw_init_trace_palette();
        self.draw_trace_segments(0);
        for y in 1..length {
            self.draw_trace_dual(y);
            self.draw_trace_palette();
            self.draw_trace_segments(y);
        }
        self.draw_boxes_dual();
    }

    fn draw_init_trace_palette(&self){
        self.gpu.gl.use_program(Some(&self.init_trace_palette));
        self.gpu.set_uniform_1i(&self.init_trace_palette, "pair_tex", 1);
        self.set_facet_uniforms(&self.init_trace_palette);
        self.gpu.set_uniform_1i(&self.init_trace_palette, "io_tex",  2);
        //self.gpu.set_uniform_1i(&self.init_trace_palette, "box_tex", 3);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().palette);
    }

    fn draw_trace_segments(&self, y: i32){
        self.gpu.gl.use_program(Some(&self.trace_segment));
        self.gpu.set_uniform_1i(&self.trace_segment, "pair_tex", 1);
        self.set_facet_uniforms(&self.trace_segment);
        self.set_arrow_uniforms(&self.trace_segment, 8);
        self.gpu.draw_at_y(&self.trace_buffer.as_ref().unwrap().trace, y, 1);
    }

    fn draw_trace_dual(&self, y: i32){
        self.gpu.gl.use_program(Some(&self.trace_dual));
        self.gpu.set_uniform_1i(&self.trace_dual, "pair_tex", 1);
        self.gpu.set_uniform_1i(&self.trace_dual, "current_segment", y);
        self.gpu.set_uniform_1i(&self.trace_dual, "trace_count", self.trace_count);
        self.set_facet_uniforms(&self.trace_dual);
        self.set_arrow_uniforms(&self.trace_dual, 8);
        self.gpu.set_uniform_1i(&self.trace_dual, "box_tex", 11);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().dual);
    }

    fn draw_trace_palette(&self){
        self.gpu.gl.use_program(Some(&self.trace_palette));
        self.gpu.set_uniform_1i(&self.trace_palette, "pair_tex", 1);
        self.set_facet_uniforms(&self.trace_palette);
        self.set_arrow_uniforms(&self.trace_palette, 4);
        self.gpu.set_uniform_1i(&self.trace_palette, "box_tex", 7);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().palette);
    }

    fn draw_boxes_dual(&self){
        self.gpu.gl.use_program(Some(&self.boxes_dual));
        self.gpu.set_uniform_1i(&self.boxes_dual, "pair_tex", 1);
        self.set_arrow_uniforms(&self.boxes_dual, 8);
        self.gpu.set_uniform_1i(&self.boxes_dual, "box_tex", 11);
        self.gpu.draw(&self.trace_buffer.as_ref().unwrap().boxes);
    }
}



        // self.facet_hits = traced_curves;
        // for traced in traced_curves{
        //     self.shapes.push(Shape::Curve(traced.center));
        // }
        // for TracedCurve{index_pair, curve0, curve1, center} in traced_curves {
        //     let TestPair3{group, i0, i1, reverse} = index_pair;
        //     self.facet_hits[group].push(Hit3{i0, i1, curve0, curve1});
        //     // self.facet_hits[index][i0].push(curve0); // [g1-g0-1]
        //     // self.facet_hits[index][i1].push(curve1); // [0]
        //     self.shapes.push(Shape::Curve(center));
        // }  
        // for MissPair{index, distance, dot0, dot1} in trace_basis.misses {
        //     let TestPair3{group, i0, i1, reverse} = index;
        //     self.facet_miss[group].push(Miss3{i0, i1, distance, dot0, dot1});
        //     // self.facet_miss[index][i0].push(Miss{distance, dot:dot0});
        //     // self.facet_miss[index][i1].push(Miss{distance, dot:dot1});
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
    //     self.gpu.set_uniform_1i(&self.hone_trace_program, "origin_tex", 2);
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
    //     self.gpu.set_uniform_1i(&self.trace_program, "origin_tex", 2);
    //     self.gpu.set_uniform_1i(&self.trace_program, "uv_tex",  7); // 5
    //     self.gpu.set_uniform_1i(&self.trace_program, "box_tex", 8); // 6
    //     self.gpu.draw(&self.trace_buffer.as_ref().unwrap().segment);
    // }
