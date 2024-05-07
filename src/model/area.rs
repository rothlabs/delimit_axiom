use crate::actor::MakeArea;
use crate::shape::*;
use crate::{Model, Models, Reshape};
use serde::*;
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Area {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
}

impl Area { 
    pub fn shapes(&self) -> Vec<Shape> {
        self.reshape.shapes(
            self.parts.shapes().area()
        )
    }
}