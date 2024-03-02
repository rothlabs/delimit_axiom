use std::f32::{INFINITY, NEG_INFINITY};
use crate::{log, get_curves, get_points, CurveShape, FacetShape, Reshape, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Area::default")]
pub struct Area {
    pub parts: Vec<Model>,
    pub reshape: Reshape,
}

impl Area { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        for point in &get_points(&self.parts) {
            shapes.push(Shape::Point(point.clone()));
        }
        let mut min = Vec3::new(INFINITY, INFINITY, INFINITY);
        let mut max = Vec3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
        let curves = get_curves(&self.parts);
        for curve in &curves {
            shapes.push(Shape::Curve(curve.clone()));
            for point in &curve.controls {
                min = min.min(*point);
                max = max.max(*point);
            }
        }
        let mut facet  = FacetShape::default();
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();

        curve0.controls.push(vec3(min.x, min.y, 0.));
        curve0.controls.push(vec3(max.x, min.y, 0.));
        curve1.controls.push(vec3(min.x, max.y, 0.));
        curve1.controls.push(vec3(max.x, max.y, 0.));
        
        facet.controls.extend([curve0, curve1]);
        for curve in &curves {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for p in boundary.controls {
                normalized_points.push(vec3(
                    (p.x - min.x) / (max.x - min.x), 
                    (p.y - min.y) / (max.y - min.y), //1. - (p.y - min.y) / (max.y - min.y), 
                    0.
                ));
            }
            boundary.controls = normalized_points;
            facet.boundaries.push(boundary); 
        }
        let valid_facet = facet.get_valid();
        //console_log!("face boundary count: {}", valid_facet.boundaries.len());
        shapes.push(Shape::Facet(valid_facet));
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