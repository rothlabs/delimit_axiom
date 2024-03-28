use crate::{get_reshapes, get_shapes, get_vec3_or, Reshape, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Mirror::default")]
pub struct Mirror {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
    pub axis:    [f32; 3], 
}

impl Mirror {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        // for shape in get_shapes(&self.parts) {
        //     let pos = vec3(
        //         (x as f32 / x_div)*x_length - x_length/2., 
        //         (y as f32 / y_div)*y_length - y_length/2., 
        //         (z as f32 / z_div)*z_length - z_length/2., 
        //     );
        //     let mat4 = Mat4::from_translation(pos);
        //     shapes.extend(get_reshapes(&self.parts, mat4));
        // }
        self.reshape.get_reshapes(shapes)
    }
}

