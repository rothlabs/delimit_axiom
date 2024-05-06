use std::f32::consts::PI;
use crate::actor;
use crate::shape::*;
use crate::{Model, Models, Reshape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] 
pub struct Revolve {
    pub parts:   Vec<Model>,
    pub center:  Vec3,
    pub axis:    Vec3,
    pub angle:   f32,
    pub reshape: Reshape,
}

impl Default for Revolve {
    fn default() -> Self {
        Self {
            parts:   vec![],
            center:  Vec3::ZERO,
            axis:    Vec3::Z,
            angle:   PI * 2.,
            reshape: Reshape::default(),
        }
    }
}

impl Revolve {
    pub fn shapes(&self) -> Vec<Shape> {
        let mat4 = self.reshape.get_matrix();
        actor::Revolve {
            shapes: self.parts.shapes(),
            center: self.center,
            axis:   self.axis,
            angle:  self.angle,
            ..Default::default()
        }.shapes().reshaped(mat4)
    }
}

