use crate::{Shape, Model, Models};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Reshape {
    pub parts:    Vec<Model>,
    pub negate:   bool,
    pub reverse:  bool,
    pub position: Vec3,//[f32; 3],
    pub rotation: Vec3,//[f32; 3],
    pub scale:    Vec3,//[f32; 3],
    pub axis:     Vec3,//[f32; 3],
    pub angle:    f32,
}

impl Default for Reshape {
    fn default() -> Self {
        Self {
            parts:    vec![],
            negate:   false,
            reverse:  false,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale:    Vec3::ONE,
            axis:     Vec3::Z,
            angle:    0.,
        }
    }
}

impl Reshape {
    pub fn shapes(&self) -> Vec<Shape> {
        self.get_reshapes(self.parts.shapes())
    }
    pub fn get_reshapes(&self, shapes: Vec<Shape>) -> Vec<Shape> {
        let mat4 = self.get_matrix();
        let mut result = vec![];
        if self.reverse {
            for shape in shapes {
                result.push(shape.reversed().reshaped(mat4));
            }
        }else{
            for shape in shapes {
                result.push(shape.reshaped(mat4));
            }
        }
        result
    }
    pub fn get_matrix(&self) -> Mat4 {
        let mut mat4 = Mat4::IDENTITY;

        //let position = get_vec3_or(&self.position, Vec3::ZERO);
        mat4 *= Mat4::from_translation(self.position);

        //let rotation = get_vec3_or(&self.rotation, Vec3::ZERO);
        if self.angle > 0. && self.axis.length() > 0. {
            //let axis = get_vec3_or(&self.axis, Vec3::Z);
            mat4 *= Mat4::from_axis_angle(self.axis, self.angle); //Mat4::from_euler(order, a, b, c)
        }else{
            mat4 *= Mat4::from_euler(EulerRot::XYZ, self.rotation.x, self.rotation.y, self.rotation.z); //Mat4::from_euler(order, a, b, c)
        }

        //let scale = get_vec3_or(&self.scale, Vec3::ONE);
        mat4 *= Mat4::from_scale(self.scale);

        mat4
    }
}

