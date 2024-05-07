use std::f32::{INFINITY, NEG_INFINITY};
use crate::shape::*;
use glam::*;

pub trait MakeArea {
    fn area(&self) -> Vec<Shape>;
}

impl MakeArea for Vec<Shape> { // TODO: make general so creates volume from facets and so on (not just facets from curves)
    fn area(&self) -> Vec<Shape> {
        let mut shapes = self.clone();
        let mut min = Vec3::new(INFINITY, INFINITY, INFINITY);
        let mut max = Vec3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
        for curve in shapes.of_rank(1) {
            for control in &curve.controls {
                min = min.min(control.point(&[]));
                max = max.max(control.point(&[]));
            }
        }
        let mut facet  = Shape::default();
        let mut curve0 = Shape::default();
        let mut curve1 = Shape::default();
        curve0.controls.push(rank0(vec3(min.x, min.y, 0.)));
        curve0.controls.push(rank0(vec3(max.x, min.y, 0.)));
        curve1.controls.push(rank0(vec3(min.x, max.y, 0.)));
        curve1.controls.push(rank0(vec3(max.x, max.y, 0.)));
        curve0.validate();
        curve1.validate();
        facet.controls.extend([curve0, curve1]);
        for curve in shapes.of_rank(1) {
            let mut boundary = curve.clone();
            let mut normalized_points = vec![];
            for bndry in boundary.controls {
                normalized_points.push(rank0(vec3(
                    (bndry.point(&[]).x - min.x) / (max.x - min.x), 
                    (bndry.point(&[]).y - min.y) / (max.y - min.y), //1. - (p.y - min.y) / (max.y - min.y), 
                    0.
                )));
            }
            boundary.controls = normalized_points;
            facet.boundaries.push(boundary); 
        }
        facet.validate();
        shapes.push(facet);
        shapes
    }
    // pub fn from_parts(parts: Vec<Model>) -> Self {
    //     let mut area = Area::default();
    //     area.parts = parts;
    //     area
    // }
    // pub fn from_part(part: Model) -> Self {
    //     let mut area = Area::default();
    //     area.parts = vec![part];
    //     area
    // }
}