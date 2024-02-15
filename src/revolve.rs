use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::{get_shapes, get_transformed_point, get_vec3_or, nurbs::Nurbs, CurveShape, FacetShape, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Revolve::default")]
pub struct Revolve {
    pub parts:  Vec<Model>,
    pub center: [f32; 3],
    pub axis:   [f32; 3],
    pub angle:  f32,
}

impl Revolve {
    pub fn get_shapes(&self) -> Vec<Shape> { // , query: &DiscreteQuery
        let center = get_vec3_or(&self.center, Vec3::ZERO);
        let axis = get_vec3_or(&self.axis, Vec3::Z).normalize(); 
        let mut basis = Basis::new(center, axis, self.angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2,    FRAC_PI_4);
        basis.add_intermediate_turn_if_needed(PI,           FRAC_PI_4*3.);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2*3., FRAC_PI_4*5.);
        basis.add_second_to_last_turn();
        let final_turn = basis.get_transform(self.angle, 1.);
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            shapes.push(shape.clone());
            match &shape {
                Shape::Point(point) => {
                    let mut curve = CurveShape {
                        nurbs: basis.nurbs.clone(),
                        controls: vec![*point], 
                        min: 0.,
                        max: 1.,
                    };
                    for &mat4 in &basis.transforms {
                        curve.controls.push(get_transformed_point(point, mat4)); 
                    }
                    curve.controls.push(get_transformed_point(point, final_turn)); 
                    shapes.push(Shape::Curve(curve));
                    shapes.push(shape.get_transformed(final_turn));
                },
                Shape::Curve(curve) => {
                    let mut facet = FacetShape {
                        nurbs: basis.nurbs.clone(),
                        controls:   vec![curve.clone()], 
                        boundaries: vec![],
                        reversed:   false,
                    };
                    for &mat4 in &basis.transforms {
                        facet.controls.push(curve.get_transformed(mat4)); 
                    }
                    facet.controls.push(curve.get_transformed(final_turn)); 
                    shapes.push(Shape::Facet(facet));
                    shapes.push(shape.get_transformed(final_turn));
                },
                Shape::Facet(facet) => {
                    shapes.push(Shape::Facet(facet.get_reversed_and_transformed(final_turn)));
                },
            }
        }
        shapes 
    }
}

struct Basis {
    nurbs: Nurbs,
    axis: Vec3,
    angle: f32,
    direction: f32,
    base_angle: f32,
    transforms: Vec<Mat4>,
    translation: Mat4,
    reverse_translation: Mat4,
}

impl Basis {
    fn new(center: Vec3, axis: Vec3, angle: f32) -> Self {
        Basis {
            nurbs: Nurbs {
                order:   3,
                knots:   vec![0.; 3],
                weights: vec![1.],
            },
            axis,
            angle: angle.abs(),
            direction: angle.signum(),
            base_angle: 0.,
            transforms: vec![],
            translation: Mat4::from_translation(center),
            reverse_translation: Mat4::from_translation(-center),
        }
    }

    fn add_intermediate_turn_if_needed(&mut self, angle0: f32, angle1: f32) {
        if self.angle > angle0 { 
            self.base_angle = angle0; 
            self.nurbs.knots.extend([angle0, angle0]);
            self.nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
            self.add_transform(angle1, FRAC_1_SQRT_2);
            self.add_transform(angle0, 1.);
        } 
    }

    fn add_second_to_last_turn(&mut self) {
         let advance = (self.angle - self.base_angle) / 2.;
         self.nurbs.knots.extend([self.angle, self.angle, self.angle]);
         self.nurbs.weights.extend([advance.cos(), 1.]);
         self.add_transform(self.base_angle + advance, advance.cos());
    }

    fn add_transform(&mut self, angle: f32, weight: f32) {
        let mat4 = self.get_transform(angle, weight);
        self.transforms.push(mat4);
    }

    // TODO: fix skew/warp from diagonal axis!!!
    fn get_transform(&mut self, angle: f32, weight: f32) -> Mat4 {
        self.translation 
        * Mat4::from_scale(Vec3::new( 
            (1./weight)*(1.-self.axis.dot(Vec3::X).abs()) + self.axis.dot(Vec3::X).abs(),
            (1./weight)*(1.-self.axis.dot(Vec3::Y).abs()) + self.axis.dot(Vec3::Y).abs(),
            (1./weight)*(1.-self.axis.dot(Vec3::Z).abs()) + self.axis.dot(Vec3::Z).abs(),
        ))
        * Mat4::from_axis_angle(self.axis, angle * self.direction)
        * self.reverse_translation
    }
}

