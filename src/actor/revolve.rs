use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::shape::*;
use glam::*;


pub trait ToRevolve {
    fn revolve(self) -> Revolve;
}

impl ToRevolve for Vec<Shape> {
    fn revolve(self) -> Revolve {
        Revolve {
            shapes: self,
            ..Default::default()
        }
    }
}

impl ToRevolve for Shape {
    fn revolve(self) -> Revolve {
        Revolve {
            shapes: vec![self],
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Revolve {
    pub shapes:  Vec<Shape>,
    pub center:  Vec3,
    pub axis:    Vec3,
    pub angle:   f32,
}

impl Default for Revolve {
    fn default() -> Self {
        Self {
            shapes:  vec![],
            center:  Vec3::ZERO,
            axis:    Vec3::Z,
            angle:   PI * 2.,
        }
    }
}

impl Revolve {
    pub fn shapes(&self) -> Vec<Shape> { 
        if self.angle == 0. {
            return self.shapes.clone();
        }
        let mut basis = RevolveBasis::new(self.center, self.axis, self.angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2,    FRAC_PI_4,    self.angle);
        basis.add_intermediate_turn_if_needed(PI,           FRAC_PI_4*3., self.angle);
        basis.add_intermediate_turn_if_needed(FRAC_PI_2*3., FRAC_PI_4*5., self.angle);
        basis.add_second_to_last_turn(self.angle);
        basis.nurbs.normalize();
        let final_turn = basis.get_matrix(self.angle, 1.);
        //let high_rank = self.shapes.high_rank();
        let mut shapes1 = vec![];
        if self.angle < PI*2. {
            shapes1 = self.shapes.clone(); // + cap_shapes; overload + operator to combine shapes
        }
        for shape0 in &self.shapes {
            let shape1 = shape0.inverted();
            let shape2 = shape1.reshaped(final_turn);
            if self.angle.abs() < PI*2. {
                shapes1.push(shape2.clone());
            }
            //if high_rank == 0 || shape0.rank < high_rank {
            if shape0.boundaries.is_empty() {
                let mut shape3 = basis.nurbs.shape(); 
                shape3.controls = vec![shape1.clone()];
                for &mat4 in &basis.turns { 
                    shape3.controls.push(shape1.reshaped(mat4)); 
                }
                shape3.controls.push(shape2);
                shape3.validate(); 
                shapes1.push(shape3);
            }
        }
        shapes1
    }
    pub fn center(&mut self, center: Vec3) -> &mut Self {
        self.center = center;
        self
    }
    pub fn axis(&mut self, axis: Vec3) -> &mut Self {
        self.axis = axis;
        self
    }
    pub fn angle(&mut self, angle: f32) -> &mut Self {
        self.angle = angle;
        self
    }
}

struct RevolveBasis {
    nurbs: Basis,
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
            nurbs: Basis {
                sign: 1.,
                order:   3,
                min: 0.,
                max: 1.,
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