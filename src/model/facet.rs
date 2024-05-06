
use glam::*;
use serde::{Deserialize, Serialize};
use crate::{Model, Models};
use crate::shape::*;


#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Facet {
    pub nurbs:      Basis,
    pub controls:   Vec<Model>,
    pub boundaries: Vec<Model>,
}

impl Facet {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shape = Shape{
            rank:       2,
            basis:      self.nurbs.clone(),
            controls:   self.controls.shapes(),
            boundaries: self.boundaries.shapes(),
            vector:     None,
            rectifier:  None,
        };
        shape.validate();
        vec![shape]
    }
}
