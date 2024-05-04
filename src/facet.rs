
use crate::space::Space;
use crate::{Shape, Model, Models};
use glam::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Facet {
    pub nurbs:      Space,
    pub controls:   Vec<Model>,
    pub boundaries: Vec<Model>,
}

impl Facet {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shape = Shape{
            rank:       2,
            space:      self.nurbs.clone(),
            controls:   self.controls.shapes(),
            boundaries: self.boundaries.shapes(),
            vector:     None,
            rectifier:  None,
        };
        shape.validate();
        vec![shape]
    }
}
