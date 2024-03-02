use std::f32::consts::PI;

use crate::{get_shapes, get_reshapes, get_vec3_or, Reshape, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "RadialPattern::default")]
pub struct RadialPattern {
    pub parts:    Vec<Model>,
    pub reshape:  Reshape,
    pub axis: [f32; 3],
    pub angle: f32,
    pub count:  usize,
}

impl RadialPattern {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let axis = get_vec3_or(&self.axis, Vec3::Z);
        let mut angle = PI * 2.;
        if self.angle.abs() > 0. {
            angle = self.angle;
        }
        let basis = RadialPatternBasis {
            axis,
            angle,
            count: self.count,
        };
        self.reshape.get_reshapes(basis.get_shapes(get_shapes(&self.parts)))
    }
}

pub struct RadialPatternBasis {
    pub axis:  Vec3,
    pub count: usize,
    pub angle: f32,
}

impl RadialPatternBasis {
    pub fn get_shapes(&self, parts: Vec<Shape>) -> Vec<Shape> {
        let mut shapes = vec![];
        for i in 0..self.count {
            let angle = PI * 2. * i as f32 / (self.count-1) as f32;
            let mat4 = Mat4::from_axis_angle(self.axis, angle);
            shapes.extend(get_reshapes(parts.clone(), mat4));
        }
        shapes
    }
}

