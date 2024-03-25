use glam::IVec2;
use web_sys::{WebGl2RenderingContext as GL, WebGlFramebuffer, WebGlTexture};
use super::texture::Texture;

pub struct Framebuffer {
    pub gl: GL,
    pub texture: Texture,
}

impl Framebuffer {
    pub fn make_empty_rgba32f(&self, tex_i: u32, size: IVec2) -> Result<WebGlFramebuffer, String> {
        let (texture, _) = self.texture.make_empty_rgba32f(tex_i, size)?;
        self.make(texture)
    }
    pub fn make_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<WebGlFramebuffer, String> {
        let (texture, _) = self.texture.make_rgba32f(tex_i, texels)?;
        self.make(texture)
    }
    fn make(&self, texture: WebGlTexture) -> Result<WebGlFramebuffer, String> {
        let buffer = self.gl.create_framebuffer().ok_or("Failed to create framebuffer")?;
        self.gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&buffer));
        self.gl.framebuffer_texture_2d(GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some(&texture), 0);
        Ok(buffer)
    }
}