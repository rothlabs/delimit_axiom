use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::shape::*;
use crate::{Model, Models, Reshape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] 
pub struct Revolve {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
    pub center:  Vec3,
    pub axis:    Vec3,
    pub angle:   f32,
}

impl Default for Revolve {
    fn default() -> Self {
        Self {
            parts:   vec![],
            reshape: Reshape::default(),
            center:  Vec3::ZERO,
            axis:    Vec3::Z,
            angle:   PI * 2.,
        }
    }
}

