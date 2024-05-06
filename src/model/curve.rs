use serde::*;
use crate::shape::*;
use crate::{Model, Models};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] 
pub struct Curve {
    pub nurbs: Basis,
    pub controls: Vec<Model>,
    pub boundaries: Vec<Model>,
    pub arrows: usize, // TEMP, for testing
}

impl Default for Curve {
    fn default() -> Self {
        Self {
            nurbs: Basis::default(),
            controls: vec![],  
            boundaries: vec![], 
            arrows: 0,
        }
    }
}

impl Curve {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut curve = Shape{
            basis: self.nurbs.clone(),
            controls: self.controls.shapes(),
            boundaries: vec![],
            rectifier: None,
            vector: None,
            rank: 1,
        };
        curve.validate();
        let mut shapes = vec![];
        if self.arrows > 0 {
            for i in 0..self.arrows {
                let mut arrow_curve = Shape::default();
                let arrow = curve.get_arrow(&[i as f32 / (self.arrows - 1) as f32]);
                arrow_curve.controls.push(Shape::from_point(arrow.point));
                arrow_curve.controls.push(Shape::from_point(arrow.point + arrow.delta));
                arrow_curve.validate();
                shapes.push(arrow_curve);
            }
        }
        shapes.push(curve);
        shapes
    }
}