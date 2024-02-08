use std::f32::{INFINITY, NEG_INFINITY};
use crate::{get_curves, get_points, Model, Nurbs};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
}

impl Area { 
    pub fn get_shapes(&self) -> Vec<Model> {
        let mut shapes = vec![];
        for point in &get_points(&self.parts) {
            shapes.push(Model::Point(point.clone()));
        }
        let mut min = Vec2::new(INFINITY, INFINITY);
        let mut max = Vec2::new(NEG_INFINITY, NEG_INFINITY);
        let curves = get_curves(&self.parts);
        for curve in &curves {
            shapes.push(Model::Curve(curve.clone()));
            for point in curve.get_controls_as_vec2() {
                min = min.min(point);
                max = max.max(point);
            }
        }
        let mut facet = Nurbs::default();
        let mut curve0 = Nurbs::default();
        let mut curve1 = Nurbs::default();
        curve0.controls.push(Model::Point([min.x, min.y, 0.]));
        curve0.controls.push(Model::Point([max.x, min.y, 0.]));
        curve1.controls.push(Model::Point([min.x, max.y, 0.]));
        curve1.controls.push(Model::Point([max.x, max.y, 0.]));
        facet.controls.extend([Model::Curve(curve0), Model::Curve(curve1)]);
        for curve in &curves {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for p in boundary.get_controls_as_vec2() {
                normalized_points.push(Model::Point([
                    (p.x - min.x) / (max.x - min.x), 
                    (p.y - min.y) / (max.y - min.y), 
                    0.
                ]));
            }
            boundary.controls = normalized_points;
            facet.boundaries.push(boundary);
        }
        shapes.push(Model::Facet(facet));
        shapes
    }
}

// curve0.controls.push(Model::Point([min.x, min.y, 0.]));
//         curve0.controls.push(Model::Point([min.x, max.y, 0.]));
//         curve1.controls.push(Model::Point([max.x, min.y, 0.]));
//         curve1.controls.push(Model::Point([max.x, max.y, 0.]));
