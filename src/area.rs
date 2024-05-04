use std::f32::{INFINITY, NEG_INFINITY};
use crate::{log, Shape, Model, ModelsToShapes, Reshape, Shapes};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
    pub reshape: Reshape,
}

impl Area { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = self.parts.shapes();
        let mut min = Vec3::new(INFINITY, INFINITY, INFINITY);
        let mut max = Vec3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
        for curve in shapes.of_rank(1) {
            for control in &curve.controls {
                min = min.min(control.get_point(&[]));
                max = max.max(control.get_point(&[]));
            }
        }
        let mut facet  = Shape::default();
        let mut curve0 = Shape::default();
        let mut curve1 = Shape::default();
        curve0.controls.push(Shape::from_point(vec3(min.x, min.y, 0.)));
        curve0.controls.push(Shape::from_point(vec3(max.x, min.y, 0.)));
        curve1.controls.push(Shape::from_point(vec3(min.x, max.y, 0.)));
        curve1.controls.push(Shape::from_point(vec3(max.x, max.y, 0.)));
        curve0.validate();
        curve1.validate();
        facet.controls.extend([curve0, curve1]);
        for curve in shapes.of_rank(1) {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for bndry in boundary.controls {
                normalized_points.push(Shape::from_point(vec3(
                    (bndry.get_point(&[]).x - min.x) / (max.x - min.x), 
                    (bndry.get_point(&[]).y - min.y) / (max.y - min.y), //1. - (p.y - min.y) / (max.y - min.y), 
                    0.
                )));
            }
            boundary.controls = normalized_points;
            facet.boundaries.push(boundary); 
        }
        facet.validate();
        shapes.push(facet);
        self.reshape.get_reshapes(shapes)
    }
    pub fn from_parts(parts: Vec<Model>) -> Self {
        let mut area = Area::default();
        area.parts = parts;
        area
    }
    pub fn from_part(part: Model) -> Self {
        let mut area = Area::default();
        area.parts = vec![part];
        area
    }
}