use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::{log, nurbs::Nurbs, CurveShape, Model, ModelsToShapes, Reshape, Shapes};
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
    pub center: Vec3,
    pub axis:   Vec3,
    pub angle:  f32,
}

impl Default for Revolve {
    fn default() -> Self {
        Self {
            parts: vec![],
            reshape: Reshape::default(),
            center:  Vec3::ZERO,
            axis:    Vec3::Z,
            angle:   PI * 2.,
        }
    }
}

impl Revolve {
    pub fn get_shapes(&self) -> Vec<CurveShape> { // , query: &DiscreteQuery
        let shapes0 = self.parts.shapes();
        if self.angle == 0. {
            return shapes0;
        }
        let mut basis = RevolveBasis::new(self.center, self.axis, self.angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2,    FRAC_PI_4,    self.angle);
        basis.add_intermediate_turn_if_needed(PI,           FRAC_PI_4*3., self.angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2*3., FRAC_PI_4*5., self.angle);
        basis.add_second_to_last_turn(self.angle);
        basis.nurbs.normalize();
        let final_turn = basis.get_matrix(self.angle, 1.);
        let high_rank = shapes0.high_rank();
        let mut shapes1 = vec![];
        if self.angle < PI*2. {
            shapes1 = shapes0.clone(); // + cap_shapes; overload + operator to combine shapes
        }
        for shape0 in shapes0 {
            let mut shape1 = shape0.clone();
            shape1 = shape1.invert().reshaped(final_turn);
            if self.angle.abs() < PI*2. {
                shapes1.push(shape1.clone());
            }
            if high_rank == 0 || shape0.rank < high_rank {
                let mut shape2 = basis.nurbs.shape(); 
                shape2.controls = vec![shape0.clone()];
                for &mat4 in &basis.turns { 
                    shape2.controls.push(shape0.reshaped(mat4)); 
                }
                shape2.controls.push(shape1);
                shape2.validate(); 
                shapes1.push(shape2);
            }
        }
        self.reshape.get_reshapes(shapes1) 
    }
}

struct RevolveBasis {
    nurbs: Nurbs,
    axis: Vec3,
    direction: f32,
    base_angle: f32,
    turns: Vec<Mat4>,
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
            direction: angle.signum(),
            base_angle: 0.,
            turns: vec![],
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
        self.turns.push(mat4);
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





// match &shape {
//     Shape::Point(point) => {
//         // let mut curve = CurveShape {
//         //     nurbs: basis.nurbs.clone(),
//         //     controls: vec![*point], 
//         //     min: 0.,
//         //     max: 1.,
//         // };
//         let mut curve = CurveShape::from_nurbs_and_controls(
//             basis.nurbs.clone(), 
//             vec![*point]
//         );
//         for &mat4 in &basis.transforms {
//             curve.controls.push(get_reshaped_point(point, mat4)); 
//         }
//         curve.controls.push(get_reshaped_point(point, final_turn)); 
//         //curve.controls.reverse();
//         shapes.push(Shape::Curve(curve));
//         //if angle.abs() < PI*2. {
//             shapes.push(shape.clone().get_reshape(final_turn));
//         //}
//     },
//     Shape::Curve(curve) => {
//         let mut facet = FacetShape {
//             nurbs: basis.nurbs.clone(),
//             controls:   vec![curve.clone()], 
//             boundaries: Rectangle::unit(),
//         };
//         for &mat4 in &basis.transforms {
//             facet.controls.push(curve.reshaped(mat4)); 
//         }
//         facet.controls.push(curve.reshaped(final_turn)); 
//         facet.controls.reverse();
//         shapes.push(Shape::Facet(facet));
//         if angle.abs() < PI*2. {
//             shapes.push(shape.clone().get_reshape(final_turn));
//         }
//     },
//     Shape::Facet(facet) => {
//         shapes.push(Shape::Facet(facet.get_reverse_reshape(final_turn)));
//     },
// }
// }
// self.reshape.get_reshapes(shapes) 