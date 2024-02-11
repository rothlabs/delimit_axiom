use std::f32::{INFINITY, NEG_INFINITY};
use crate::{Model, Shape, Nurbs, Boundary, get_curves, get_points};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
}

impl Area { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        for point in &get_points(&self.parts) {
            shapes.push(Shape::Point(point.clone()));
        }
        let mut min = Vec2::new(INFINITY, INFINITY);
        let mut max = Vec2::new(NEG_INFINITY, NEG_INFINITY);
        let curves = get_curves(&self.parts);
        for curve in &curves {
            shapes.push(Shape::Curve(curve.clone()));
            for point in curve.get_controls_as_vec2() {
                min = min.min(point);
                max = max.max(point);
            }
        }
        let mut facet = Nurbs::default();
        let mut curve0 = Nurbs::default();
        let mut curve1 = Nurbs::default();

        curve0.controls.push(Shape::Point([min.x, max.y, 0.]));
        curve0.controls.push(Shape::Point([max.x, max.y, 0.]));
        curve1.controls.push(Shape::Point([min.x, min.y, 0.]));
        curve1.controls.push(Shape::Point([max.x, min.y, 0.]));
        
        facet.controls.extend([Shape::Curve(curve0), Shape::Curve(curve1)]);
        for curve in &curves {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for p in boundary.get_controls_as_vec2() {
                normalized_points.push(Shape::Point([
                    (p.x - min.x) / (max.x - min.x), 
                    1. - (p.y - min.y) / (max.y - min.y), 
                    0.
                ]));
            }
            boundary.controls = normalized_points;
            facet.boundaries.push(Boundary::Curve(boundary));
        }
        shapes.push(Shape::Facet(facet));
        shapes
    }
}