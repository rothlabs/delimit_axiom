pub mod shader;
pub mod texture;
pub mod framebuffer;
use glam::{IVec2};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlTexture};
use self::framebuffer::Framebuffer;
use self::shader::{PASS_VERTEX_SOURCE, COPY_FRAGMENT_SOURCE};
use self::texture::Texture;

pub struct GPU {
    pub gl: GL,
    pub pass_vertex_shader: Option<WebGlShader>,
    pub texture: Texture,
    pub framebuffer: Framebuffer,
}

impl GPU {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("compute_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let gl = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;
        gl.get_extension("EXT_color_buffer_float").expect("EXT_color_buffer_float extension required");
        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        let quad_points: [f32; 12] = [-1.,-1.,  1.,-1.,  -1.,1.,  -1.,1.,  1.,-1.,  1.,1.];
        unsafe {
            gl.buffer_data_with_array_buffer_view(
                GL::ARRAY_BUFFER,
                &js_sys::Float32Array::view(&quad_points),
                GL::STATIC_DRAW,
            );
        }
        let texture = Texture{gl:gl.clone()};
        let mut gpu = Self {
            gl: gl.clone(), 
            pass_vertex_shader: None, 
            texture: texture.clone(),
            framebuffer: Framebuffer{gl, texture},
        };
        gpu.pass_vertex_shader = Some(gpu.get_vertex_shader(PASS_VERTEX_SOURCE)?);
        let program = gpu.get_quad_program_from_source(COPY_FRAGMENT_SOURCE)?;
        let position_attribute_location = gpu.gl.get_attrib_location(&program, "position");
        let vao = gpu.gl.create_vertex_array().ok_or("Could not create vertex array object")?;
        gpu.gl.bind_vertex_array(Some(&vao));
        gpu.gl.vertex_attrib_pointer_with_i32(position_attribute_location as u32, 2, GL::FLOAT, false, 0, 0);
        gpu.gl.enable_vertex_attrib_array(position_attribute_location as u32);
        Ok(gpu)
    }
    pub fn get_vertex_shader(&self, source: &str) -> Result<WebGlShader, String> {
        self.get_shader(GL::VERTEX_SHADER, source)
    }
    pub fn get_fragment_shader(&self, source: &str) -> Result<WebGlShader, String> {
        self.get_shader(GL::FRAGMENT_SHADER, source)
    }
    pub fn get_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = self.gl.create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.gl.shader_source(&shader, source);
        self.gl.compile_shader(&shader);
        if self.gl
            .get_shader_parameter(&shader, GL::COMPILE_STATUS)
            .as_bool().unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(self.gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }
    pub fn get_quad_program_from_source(&self, source: &str) -> Result<WebGlProgram, String> {
        let shader = self.get_shader(GL::FRAGMENT_SHADER, source)?;
        Ok(self.get_quad_program(&shader)?)
    }
    pub fn get_quad_program(&self, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
        self.get_program(self.pass_vertex_shader.as_ref().unwrap(), frag_shader)
    }
    pub fn get_program(&self, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
        let program = self.gl.create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.gl.attach_shader(&program, vert_shader);
        self.gl.attach_shader(&program, frag_shader);
        self.gl.link_program(&program);
        if self.gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool().unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(self.gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }
}


// fn set_texture_parameters(&self) {
//     self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
//     self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
//     self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
//     self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
// }


// impl GPU {
//     pub fn new() -> Result<Self, JsValue> {
//         let document = web_sys::window().unwrap().document().unwrap();
//         let canvas = document.get_element_by_id("compute_canvas").unwrap();
//         let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
//         let gl = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;
//         let mut gpu = Self{gl, pass_vertex_shader:None};
//         gpu.pass_vertex_shader = Some(gpu.compile_shader(
//             GL::VERTEX_SHADER,
//             r##"#version 300 es
//             in vec4 position;
//             void main() {
//                 gl_Position = position;
//             }
//             "##,
//         )?);
//         Ok(gpu)
//     }
//     pub fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
//         let shader = self.gl.create_shader(shader_type)
//             .ok_or_else(|| String::from("Unable to create shader object"))?;
//         self.gl.shader_source(&shader, source);
//         self.gl.compile_shader(&shader);
//         if self.gl
//             .get_shader_parameter(&shader, GL::COMPILE_STATUS)
//             .as_bool().unwrap_or(false)
//         {
//             Ok(shader)
//         } else {
//             Err(self.gl.get_shader_info_log(&shader)
//                 .unwrap_or_else(|| String::from("Unknown error creating shader")))
//         }
//     }
//     pub fn make_fragment_program_from_source(&self, source: &str) -> Result<WebGlProgram, String> {
//         let shader = self.compile_shader(GL::FRAGMENT_SHADER, source)?;
//         Ok(self.make_fragment_program(&shader)?)
//     }
//     pub fn make_fragment_program(&self, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
//         self.make_program(self.pass_vertex_shader.as_ref().unwrap(), frag_shader)
//     }
//     pub fn make_program(&self, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
//         let program = self.gl.create_program()
//             .ok_or_else(|| String::from("Unable to create shader object"))?;
//         self.gl.attach_shader(&program, vert_shader);
//         self.gl.attach_shader(&program, frag_shader);
//         self.gl.link_program(&program);
//         if self.gl
//             .get_program_parameter(&program, GL::LINK_STATUS)
//             .as_bool().unwrap_or(false)
//         {
//             Ok(program)
//         } else {
//             Err(self.gl
//                 .get_program_info_log(&program)
//                 .unwrap_or_else(|| String::from("Unknown error creating program object")))
//         }
//     }
// }



// let document = web_sys::window().unwrap().document().unwrap();
        // let canvas = document.get_element_by_id("compute_canvas").unwrap();
        // let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        // let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<GL>().unwrap();