use crate::{get_shapes, get_transformed_point, get_vec3_or, nurbs::Nurbs, CurveShape, Facet, FacetShape, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Extrude::default")]
pub struct Extrude {
    pub parts:  Vec<Model>,
    pub axis:   [f32; 3],
    pub length: f32,
}

impl Extrude {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let axis = get_vec3_or(&self.axis, Vec3::Z).normalize(); 
        let basis = ExtrudeBasis::new(axis * self.length);
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            if let Shape::Facet(facet) = &shape {
                shapes.push(Shape::Facet(facet.get_transformed_and_reversed(Mat4::IDENTITY)));
            }else{
                shapes.push(shape.clone());
            }
            match &shape {
                Shape::Point(point) => {
                    let mut curve = CurveShape {
                        nurbs: basis.nurbs.clone(),
                        controls: vec![get_transformed_point(point, basis.mat4)], 
                        min: 0.,
                        max: 1.,
                    };
                    curve.controls.push(*point);
                    shapes.push(Shape::Curve(curve));
                    shapes.push(shape.get_transformed(basis.mat4));
                },
                Shape::Curve(curve) => {
                    let mut facet = FacetShape {
                        nurbs: basis.nurbs.clone(),
                        controls:   vec![curve.get_transformed(basis.mat4)], 
                        boundaries: vec![],
                        //reversed:   false,
                        perimeter:  false,
                    };
                    facet.controls.push(curve.clone()); 
                    shapes.push(Shape::Facet(facet));
                    shapes.push(shape.get_transformed(basis.mat4));
                },
                Shape::Facet(facet) => {
                    shapes.push(Shape::Facet(facet.get_transformed(basis.mat4)));
                },
            }
        }
        shapes 
    }
}

struct ExtrudeBasis {
    nurbs: Nurbs,
    mat4: Mat4,
}

impl ExtrudeBasis {
    fn new(translation: Vec3) -> Self {
        Self {
            nurbs: Nurbs {
                order:   2,
                knots:   vec![0., 0., 1., 1.],
                weights: vec![1., 1.],
            },
            mat4: Mat4::from_translation(translation),
        }
    }
}

