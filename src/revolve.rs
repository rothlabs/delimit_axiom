use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::{get_reshaped_point, get_shapes, get_vec3_or, nurbs::Nurbs, Curve, FacetShape, Reshape, Model, Rectangle, Shape};
use serde::{Deserialize, Serialize};
use glam::*;

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] //  = "Revolve::default"
pub struct Revolve {
    pub parts:  Vec<Model>,
    pub reshape: Reshape,
    pub center: Vec3,//[f32; 3],
    pub axis:   Vec3,//[f32; 3],
    pub angle:  f32,
}

impl Default for Revolve {
    fn default() -> Self {
        Self {
            parts: vec![],
            reshape: Reshape::default(),
            center:  Vec3::ZERO,
            axis:    Vec3::Z,
            angle:   PI,
        }
    }
}

impl Revolve {
    pub fn get_shapes(&self) -> Vec<Shape> { // , query: &DiscreteQuery
        let mut angle = self.angle;
        if angle == 0. {angle = PI*2.};
        let mut basis = RevolveBasis::new(self.center, self.axis, angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2,    FRAC_PI_4,    angle);
        basis.add_intermediate_turn_if_needed(PI,           FRAC_PI_4*3., angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2*3., FRAC_PI_4*5., angle);
        basis.add_second_to_last_turn(angle);
        basis.nurbs.normalize_knots();
        let final_turn = basis.get_matrix(angle, 1.);
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            if angle.abs() < PI*2. {
                shapes.push(shape.clone());
            }
            match &shape {
                Shape::Point(point) => {
                    let mut curve = Curve {
                        nurbs: basis.nurbs.clone(),
                        controls: vec![*point], 
                        min: 0.,
                        max: 1.,
                    };
                    for &mat4 in &basis.transforms {
                        curve.controls.push(get_reshaped_point(point, mat4)); 
                    }
                    curve.controls.push(get_reshaped_point(point, final_turn)); 
                    //curve.controls.reverse();
                    shapes.push(Shape::Curve(curve));
                    //if angle.abs() < PI*2. {
                        shapes.push(shape.clone().get_reshape(final_turn));
                    //}
                },
                Shape::Curve(curve) => {
                    let mut facet = FacetShape {
                        nurbs: basis.nurbs.clone(),
                        controls:   vec![curve.clone()], 
                        boundaries: Rectangle::unit(),
                    };
                    for &mat4 in &basis.transforms {
                        facet.controls.push(curve.get_reshape(mat4)); 
                    }
                    facet.controls.push(curve.get_reshape(final_turn)); 
                    facet.controls.reverse();
                    shapes.push(Shape::Facet(facet));
                    if angle.abs() < PI*2. {
                        shapes.push(shape.clone().get_reshape(final_turn));
                    }
                },
                Shape::Facet(facet) => {
                    shapes.push(Shape::Facet(facet.get_reverse_reshape(final_turn)));
                },
            }
        }
        self.reshape.get_reshapes(shapes) 
    }
}

struct RevolveBasis {
    nurbs: Nurbs,
    axis: Vec3,
    angle: f32,
    direction: f32,
    base_angle: f32,
    transforms: Vec<Mat4>,
    translation: Mat4,
    reverse_translation: Mat4,
}

impl RevolveBasis {
    fn new(center: Vec3, axis: Vec3, angle: f32) -> Self {
        Self {
            nurbs: Nurbs {
                sign: 1.,
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

    fn add_intermediate_turn_if_needed(&mut self, angle0: f32, angle1: f32, input_angle: f32) {
        if input_angle > angle0 { 
            self.base_angle = angle0; 
            self.nurbs.knots.extend([angle0, angle0]);
            self.nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
            self.add_transform(angle1, FRAC_1_SQRT_2);
            self.add_transform(angle0, 1.);
        } 
    }

    fn add_second_to_last_turn(&mut self, input_angle: f32) {
         let advance = (input_angle - self.base_angle) / 2.;
         self.nurbs.knots.extend([input_angle, input_angle, input_angle]);
         self.nurbs.weights.extend([advance.cos(), 1.]);
         self.add_transform(self.base_angle + advance, advance.cos());
    }

    fn add_transform(&mut self, angle: f32, weight: f32) {
        let mat4 = self.get_matrix(angle, weight);
        self.transforms.push(mat4);
    }

    // TODO: fix skew/warp from diagonal axis!!!
    fn get_matrix(&mut self, angle: f32, weight: f32) -> Mat4 {
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

