use glam::IVec2;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext as GL, WebGlFramebuffer, WebGlTexture};
use super::texture::TextureContext;

pub struct Framebuffer {
    pub content: WebGlFramebuffer,
    pub size: IVec2,
}

#[derive(Clone)]
pub struct FramebufferContext {
    pub gl: GL,
    pub texture: TextureContext,
}

impl FramebufferContext {
    pub fn make_row_rgba32f(&self, tex_i: u32, texel_groups: &mut Vec<Vec<f32>>) -> Result<Framebuffer, String> {
        let mut textures = vec![];
        let mut size = IVec2::ZERO;
        for i in 0..texel_groups.len() {
            let (texture, new_size) = self.texture.make_row_rgba32f(tex_i + i as u32, &mut texel_groups[i])?;
            textures.push(texture);
            size = new_size;
        } 
        self.make(textures, size)
    }
    pub fn make_multi_empty_rgba32f(&self, tex_i: u32, size: IVec2, count: usize) -> Result<Framebuffer, String> {
        let mut textures = vec![];
        for i in 0..count as u32{
            textures.push(self.texture.make_empty_rgba32f(tex_i + i, size)?);
        } 
        self.make(textures, size)
    }
    pub fn make_rgba32f_with_empties(&self, tex_i: u32, texels: &mut Vec<f32>, count: usize) -> Result<Framebuffer, String> {
        let (texture, size) = self.texture.make_rgba32f(tex_i, texels)?;
        let mut textures = vec![texture];
        for i in 1..count as u32{
            textures.push(self.texture.make_empty_rgba32f(tex_i + i, size)?);
        } 
        self.make(textures, size)
    }
    pub fn make_empty_rgba32f(&self, tex_i: u32, size: IVec2) -> Result<Framebuffer, String> {
        let texture = self.texture.make_empty_rgba32f(tex_i, size)?;
        self.make(vec![texture], size)
    }
    pub fn make_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<Framebuffer, String> {
        let (texture, size) = self.texture.make_rgba32f(tex_i, texels)?;
        self.make(vec![texture], size)
    }
    fn make(&self, textures: Vec<WebGlTexture>, size:IVec2) -> Result<Framebuffer, String> {
        let buffer = self.gl.create_framebuffer().ok_or("Failed to create framebuffer")?;
        self.gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&buffer));
        //self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&texture), 0);
        let attachments = js_sys::Array::new(); // vec![];
        for (i, tex) in textures.iter().enumerate() {
            let attachment = GL::COLOR_ATTACHMENT0 + i as u32;
            attachments.push(&JsValue::from(attachment));
            self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, attachment, GL::TEXTURE_2D, Some(&tex), 0);
        }
        self.gl.draw_buffers(&attachments);
        Ok(Framebuffer{content:buffer, size})
    }
}

//let js_attachments = JsValue::from(attachments.into_iter().map(|x| JsValue::from(x)).collect::<js_sys::Array>());

// pub fn make_row_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>, extra: usize) -> Result<Framebuffer, String> {
//     let (texture, size) = self.texture.make_row_rgba32f(tex_i, texels)?;
//     self.make(texture, size, tex_i, extra)
// }

// pub fn make_empty_rgba32f(&self, tex_i: u32, size: IVec2, extra: usize) -> Result<Framebuffer, String> {
//     let (texture, _) = self.texture.make_empty_rgba32f(tex_i, size)?;
//     self.make(texture, size, tex_i, extra)
// }
// pub fn make_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>, extra: usize) -> Result<Framebuffer, String> {
//     let (texture, size) = self.texture.make_rgba32f(tex_i, texels)?;
//     self.make(texture, size, tex_i, extra)
// }
// fn make(&self, texture: WebGlTexture, size:IVec2, tex_i: u32, extra: usize) -> Result<Framebuffer, String> {
//     let buffer = self.gl.create_framebuffer().ok_or("Failed to create framebuffer")?;
//     self.gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&buffer));
//     self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&texture), 0);
//     let mut attachments = vec![GL::COLOR_ATTACHMENT0];
//     for i in 0..extra as u32 {
//         let (tex, _) = self.texture.make_empty_rgba32f(tex_i + 1 + i, size)?;
//         let attachment = GL::COLOR_ATTACHMENT0 + 1 + i;
//         attachments.push(attachment);
//         self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, attachment, GL::TEXTURE_2D, Some(&tex), 0);
//     }
//     let js_attachments = JsValue::from(attachments.into_iter().map(|x| JsValue::from(x)).collect::<js_sys::Array>());
//     self.gl.draw_buffers(&js_attachments);
//     Ok(Framebuffer{content:buffer, size})
// }