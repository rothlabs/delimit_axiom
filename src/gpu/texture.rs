use glam::{IVec2, ivec2};
use web_sys::{WebGl2RenderingContext as GL, WebGlTexture};

#[derive(Clone)]
pub struct TextureContext {
    pub gl: GL,
}

impl TextureContext {
    pub fn make_row_rg32i(&self, tex_i: u32, texels: &mut Vec<i32>) -> Result<(WebGlTexture, IVec2), String> {
        let size = ivec2(texels.len() as i32 / 2, 1);
        self.make_with_i32_and_size(tex_i, texels, size, 2, GL::RG32I, GL::RG_INTEGER, GL::INT)
    }
    pub fn make_row_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<(WebGlTexture, IVec2), String> {
        let size = ivec2(texels.len() as i32 / 2, 1);
        self.make_with_f32_and_size(tex_i, texels, size, 4, GL::RGBA32F, GL::RGBA, GL::FLOAT)
    }

    pub fn make_rg32i(&self, tex_i: u32, texels: &mut Vec<i32>) -> Result<(WebGlTexture, IVec2), String> {
        let size = get_size(texels.len(), 2);
        self.make_with_i32_and_size(tex_i, texels, size, 2, GL::RG32I, GL::RG_INTEGER, GL::INT)
    }
    pub fn make_empty_rgba32f(&self, tex_i: u32, size: IVec2) -> Result<(WebGlTexture, IVec2), String> {
        self.make_empty(tex_i, size, GL::RGBA32F, GL::RGBA, GL::FLOAT)
    }
    pub fn make_rgba32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<(WebGlTexture, IVec2), String> {
        let size = get_size(texels.len(), 4);
        self.make_with_f32_and_size(tex_i, texels, size, 4, GL::RGBA32F, GL::RGBA, GL::FLOAT)
    }
    pub fn make_r32f(&self, tex_i: u32, texels: &mut Vec<f32>) -> Result<(WebGlTexture, IVec2), String> {
        let size = get_size(texels.len(), 1);
        self.make_with_f32_and_size(tex_i, texels, size, 1, GL::R32F, GL::RED, GL::FLOAT)
    }
    fn make_empty(&self, tex_i: u32, size: IVec2, internal_format: u32, format: u32, type_: u32
    ) -> Result<(WebGlTexture, IVec2), String> {
        let texture = self.make(tex_i)?;
        self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            GL::TEXTURE_2D, 0, internal_format as i32, size.x, size.y, 0, format, type_, None
        ).expect("Could not set tex_image_2d with None");
        Ok((texture, size))
    }
    fn make_with_i32_and_size(&self, tex_i: u32, texels: &mut Vec<i32>, 
        size: IVec2, item_size: i32, internal_format: u32, format: u32, type_: u32
    ) -> Result<(WebGlTexture, IVec2), String> {
        add_i32_zeros(texels, size.x*size.y*item_size);
        let texture = self.make(tex_i)?;
        unsafe{
            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                GL::TEXTURE_2D, 0, internal_format as i32, size.x, size.y, 0, format, type_, Some(&js_sys::Int32Array::view(&texels))
            ).expect("Could not set tex_image_2d with i32");
        }
        Ok((texture, size))
    }
    fn make_with_f32_and_size(&self, tex_i: u32, texels: &mut Vec<f32>, 
        size: IVec2, item_size: i32, internal_format: u32, format: u32, type_: u32
    ) -> Result<(WebGlTexture, IVec2), String> {
        add_f32_zeros(texels, size.x*size.y*item_size);
        let texture = self.make(tex_i)?;
        unsafe{
            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                GL::TEXTURE_2D, 0, internal_format as i32, size.x, size.y, 0, format, type_, Some(&js_sys::Float32Array::view(&texels))
            ).expect("Could not set tex_image_2d with f32");
        }
        Ok((texture, size))
    }
    fn make(&self, tex_i: u32) -> Result<WebGlTexture, String> {
        self.gl.active_texture(GL::TEXTURE0 + tex_i);
        let texture = self.gl.create_texture().ok_or("Failed to create texture")?;
        self.gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        self.gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        Ok(texture)
    }
}

fn get_size(count: usize, item_size: usize) -> IVec2 {
    let texel_count = count / item_size;
    let width = (texel_count as f32).sqrt().ceil();
    let height = (texel_count as f32 / width).ceil();
    ivec2(width as i32, height as i32)
}

fn add_i32_zeros(texels: &mut Vec<i32>, count: i32) {
    for _ in 0..count as usize - texels.len(){
        texels.push(0_i32);
    }
}

fn add_f32_zeros(texels: &mut Vec<f32>, count: i32) {
    for _ in 0..count as usize - texels.len(){
        texels.push(0_f32);
    }
}

// fn get_tex_enum(num: u32) -> u32 {
//     GL::TEXTURE0
//     GL::TEXTURE1
// }