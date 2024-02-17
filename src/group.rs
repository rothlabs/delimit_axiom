use crate::{Model, Shape, get_shapes, get_vec3_or};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Group::default")]
pub struct Group {
    pub parts:    Vec<Model>,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale:    [f32; 3],
    pub axis:     [f32; 3],
    pub angle:    f32,
}

impl Group {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mat4 = self.get_matrix();
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            shapes.push(shape.get_transformed(mat4));
        }
        shapes
    }
    fn get_matrix(&self) -> Mat4 {
        let mut mat4 = Mat4::IDENTITY;

        let position = get_vec3_or(&self.position, Vec3::ZERO);
        mat4 *= Mat4::from_translation(position);

        let rotation = get_vec3_or(&self.rotation, Vec3::ZERO);
        if rotation.length() > 0. {
            mat4 *= Mat4::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z); //Mat4::from_euler(order, a, b, c)
        }else{
            let axis = get_vec3_or(&self.axis, Vec3::Z);
            mat4 *= Mat4::from_axis_angle(axis, self.angle); //Mat4::from_euler(order, a, b, c)
        }

        let scale = get_vec3_or(&self.scale, Vec3::ONE);
        mat4 *= Mat4::from_scale(scale);

        mat4
    }
}

