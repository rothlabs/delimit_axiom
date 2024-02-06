use crate::{Model, get_shapes};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Group::default")]
pub struct Group {
    pub parts:    Vec<Model>,
    pub position: Box<Model>, // TODO: switch to slice?
    pub axis:     Box<Model>,
    pub angle:    f32,
    pub scale:    Box<Model>,
}

impl Group {
    pub fn get_shapes(&self) -> Vec<Model> {
        let mat4 = self.get_matrix();
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            shapes.push(shape.get_transformed(mat4));
        }
        shapes
    }
    fn get_matrix(&self) -> Mat4 {
        let mut mat4 = Mat4::IDENTITY;
        let position = self.position.get_vec3_or(Vec3::ZERO);
        mat4 *= Mat4::from_translation(position);
        let axis = self.axis.get_vec3_or(Vec3::Z);
        mat4 *= Mat4::from_axis_angle(axis, self.angle);
        let scale = self.scale.get_vec3_or(Vec3::ONE);
        mat4 *= Mat4::from_scale(scale);
        mat4
    }
}

