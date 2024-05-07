use crate::{Shape, Model, Models};
use serde::*;
use glam::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Reshape {
    pub parts:    Vec<Model>,
    pub negate:   bool,
    pub reverse:  bool,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale:    Vec3,
    pub axis:     Vec3,
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
    pub fn shapes(&self, shapes: Vec<Shape>) -> Vec<Shape> {
        if shapes.len() > 0 {
            self.reshape(shapes)
        } else {
            self.reshape(self.parts.shapes())
        }
    }
    fn reshape(&self, shapes: Vec<Shape>) -> Vec<Shape> {
        let mat4 = self.matrix();
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
    pub fn matrix(&self) -> Mat4 {
        let mut mat4 = Mat4::IDENTITY;
        mat4 *= Mat4::from_translation(self.position);
        if self.angle > 0. && self.axis.length() > 0. {
            mat4 *= Mat4::from_axis_angle(self.axis, self.angle); 
        }else{
            mat4 *= Mat4::from_euler(EulerRot::XYZ, self.rotation.x, self.rotation.y, self.rotation.z); 
        }
        mat4 *= Mat4::from_scale(self.scale);

        mat4
    }
}

