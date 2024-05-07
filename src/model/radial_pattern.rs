use std::f32::consts::PI;
use crate::{Shape, Model, Models, Reshape, Shapes};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RadialPattern {
    pub parts:    Vec<Model>,
    pub reshape:  Reshape,
    pub axis:   Vec3,//[f32; 3],
    pub angle:  f32,
    pub count:  usize,
}

impl Default for RadialPattern {
    fn default() -> Self {
        Self {
            parts: vec![],
            reshape: Reshape::default(),
            axis:  Vec3::Z,
            angle: PI*2.,
            count: 2,
        }
    }
}

impl RadialPattern {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let reshape_matrix = self.reshape.matrix();
        let basis_shapes = self.parts.shapes();
        for i in 0..self.count {
            let angle = self.angle * i as f32 / self.count as f32;
            let mat4 = Mat4::from_axis_angle(self.axis, angle);
            shapes.extend(basis_shapes.reshaped(reshape_matrix * mat4));
        }
        shapes //self.reshape.get_reshapes(shapes)
    }
}

// impl RadialPattern {
//     pub fn get_shapes(&self) -> Vec<Shape> {
//         //let axis = get_vec3_or(&self.axis, Vec3::Z);
//         let mut angle = PI * 2.;
//         if self.angle.abs() > 0. {
//             angle = self.angle;
//         }
//         let basis = RadialPatternBasis {
//             axis,
//             angle,
//             count: self.count,
//         };
//         self.reshape.get_reshapes(basis.get_shapes(get_shapes(&self.parts)))
//     }
// }

// pub struct RadialPatternBasis {
//     pub axis:  Vec3,
//     pub count: usize,
//     pub angle: f32,
// }

// impl RadialPatternBasis {
//     pub fn get_shapes(&self, parts: Vec<Shape>) -> Vec<Shape> {
//         let mut shapes = vec![];
//         for i in 0..self.count {
//             let angle = self.angle * i as f32 / self.count as f32;
//             let mat4 = Mat4::from_axis_angle(self.axis, angle);
//             shapes.extend(get_reshapes(&parts, mat4));
//         }
//         shapes
//     }
// }

