use crate::{Model, Nurbs, get_curves};

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
        let mut min = Vec2::new(std::f32::INFINITY, std::f32::INFINITY);
        let mut max = Vec2::new(std::f32::NEG_INFINITY, std::f32::NEG_INFINITY);
        let curves = get_curves(&self.parts);
        for curve in &curves {
            for point in curve.get_controls_as_vec2() {
                min = min.min(point);
                max = max.max(point);
            }
        }
        let mut facet = Nurbs::default();
        let mut curve0 = Nurbs::default();
        let mut curve1 = Nurbs::default();
        curve0.controls.push(Model::Point([min.x, min.y, 0.]));
        curve0.controls.push(Model::Point([min.x, max.y, 0.]));
        curve1.controls.push(Model::Point([max.x, min.y, 0.]));
        curve1.controls.push(Model::Point([max.x, max.y, 0.]));
        facet.controls.extend([Model::Curve(curve0), Model::Curve(curve1)]);
        for curve in &curves {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for p in boundary.get_controls_as_vec2() {
                normalized_points.push(Model::Point([
                    (p.x - min.x) / max.x, 
                    (p.y - min.y) / max.y, 
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
