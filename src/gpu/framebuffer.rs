use glam::IVec2;
use web_sys::{WebGl2RenderingContext as GL, WebGlFramebuffer, WebGlTexture};
use super::texture::TextureContext;

pub struct Framebuffer {
    pub content: WebGlFramebuffer,
    pub size: IVec2,
}

pub struct FramebufferContext {
    pub gl: GL,
    pub texture: TextureContext,
}

impl FramebufferContext {
    pub fn make_row_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<Framebuffer, String> {
        let (texture, size) = self.texture.make_row_rgba32f(tex_i, texels)?;
        self.make(texture, size)
    }
    pub fn make_empty_rgba32f(&self, tex_i: u32, size: IVec2) -> Result<Framebuffer, String> {
        let (texture, _) = self.texture.make_empty_rgba32f(tex_i, size)?;
        self.make(texture, size)
    }
    pub fn make_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<Framebuffer, String> {
        let (texture, size) = self.texture.make_rgba32f(tex_i, texels)?;
        self.make(texture, size)
    }
    fn make(&self, texture: WebGlTexture, size:IVec2) -> Result<Framebuffer, String> {
        let buffer = self.gl.create_framebuffer().ok_or("Failed to create framebuffer")?;
        self.gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&buffer));
        self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&texture), 0);
        Ok(Framebuffer{content:buffer, size})
    }
}