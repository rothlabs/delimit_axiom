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
        //let angle = self.angle.abs();
        let mut basis = Basis::new(center, axis, self.angle);
        // let basis = Basis {
        //     axis,
        //     translation: Mat4::from_translation(center),
        //     reverse_translation: Mat4::from_translation(-center),
        //     direction: self.angle.signum(),
        //     nurbs: Nurbs {
        //         order:   3,
        //         knots:   vec![0.; 3],
        //         weights: vec![1.],
        //     },
        // };
        // let mut nurbs = Nurbs {
        //     order:   3,
        //     knots:   vec![0.; 3],
        //     weights: vec![1.],
        // };
        //let mut transforms = vec![];
        //let mut base_angle = 0.;

        // let mut add_turn_if_needed = |angle0, angle1| {
        //     if angle > angle0 { 
        //         base_angle = angle0; 
        //         nurbs.knots.extend([base_angle, base_angle]);
        //         nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
        //         transforms.push(basis.get_transform(angle1, FRAC_1_SQRT_2));
        //         transforms.push(basis.get_transform(base_angle, 1.));
        //     } 
        // };

        basis.add_turn_if_needed(FRAC_PI_2,    FRAC_PI_4);
        basis.add_turn_if_needed(PI,           FRAC_PI_4*3.);
        basis.add_turn_if_needed(FRAC_PI_2*3., FRAC_PI_4*5.);
        basis.add_second_to_last_turn();

        // // final turn 
        // let advance = (angle - base_angle) / 2.;
        // nurbs.knots.extend([angle, angle, angle]);
        // nurbs.weights.extend([advance.cos(), 1.]);
        // transforms.push(basis.get_transform(base_angle + advance, advance.cos()));
        let end_mat4 = basis.get_transform(self.angle, 1.); //  * self.angle.signum()

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
                    curve.controls.push(get_transformed_point(point, end_mat4)); 
                    shapes.push(Shape::Curve(curve));
                    shapes.push(shape.get_transformed(end_mat4));
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
                    facet.controls.push(curve.get_transformed(end_mat4)); 
                    shapes.push(Shape::Facet(facet));
                    shapes.push(shape.get_transformed(end_mat4));
                },
                Shape::Facet(facet) => {
                    shapes.push(Shape::Facet(facet.get_reversed_and_transformed(end_mat4)));
                },
            }
        }
        shapes 
    }
}

struct Basis {
    axis: Vec3,
    translation: Mat4,
    reverse_translation: Mat4,
    direction: f32,
    nurbs: Nurbs,
    transforms: Vec<Mat4>,
    base_angle: f32,
    angle: f32,
}


impl Basis {
    fn new(center: Vec3, axis: Vec3, angle: f32) -> Self {
        Basis {
            nurbs: Nurbs {
                order:   3,
                knots:   vec![0.; 3],
                weights: vec![1.],
            },
            translation: Mat4::from_translation(center),
            reverse_translation: Mat4::from_translation(-center),
            axis,
            angle: angle.abs(),
            direction: angle.signum(),
            transforms: vec![],
            base_angle: 0.,
            
        }
    }

    fn add_turn_if_needed(&mut self, angle0: f32, angle1: f32) {
        if self.angle > angle0 { 
            self.base_angle = angle0; 
            self.nurbs.knots.extend([angle0, angle0]);
            self.nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
            self.add_transform(angle1, FRAC_1_SQRT_2);
            self.add_transform(angle0, 1.);
            // self.transforms.push(self.get_transform(angle1, FRAC_1_SQRT_2));
            // self.transforms.push(self.get_transform(angle0, 1.));
        } 
    }

    fn add_second_to_last_turn(&mut self) {
         let advance = (self.angle - self.base_angle) / 2.;
         self.nurbs.knots.extend([self.angle, self.angle, self.angle]);
         self.nurbs.weights.extend([advance.cos(), 1.]);
         self.add_transform(self.base_angle + advance, advance.cos());
         //self.transforms.push(self.get_transform(self.base_angle + advance, advance.cos()));
         //let end_mat4 = self.get_transform(self.angle * self.angle.signum(), 1.);
    }

    fn add_transform(&mut self, angle: f32, weight: f32) {
        let mat4 = self.get_transform(angle, weight);
        self.transforms.push(mat4);
    }

    // TODO: fix skew/warp from diagonal axis!!!
    fn get_transform(&mut self, angle: f32, weight: f32) -> Mat4 {
        //self.transforms.push(
            self.translation 
            * Mat4::from_scale(Vec3::new( 
                (1./weight)*(1.-self.axis.dot(Vec3::X).abs()) + self.axis.dot(Vec3::X).abs(),
                (1./weight)*(1.-self.axis.dot(Vec3::Y).abs()) + self.axis.dot(Vec3::Y).abs(),
                (1./weight)*(1.-self.axis.dot(Vec3::Z).abs()) + self.axis.dot(Vec3::Z).abs(),
            ))
            * Mat4::from_axis_angle(self.axis, angle * self.direction)
            * self.reverse_translation
        //)
    }
}



        // // over quarter turn 
        // if angle > FRAC_PI_2 { 
        //     base_angle = FRAC_PI_2; 
        //     nurbs.knots.extend([base_angle, base_angle]);
        //     nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
        //     transforms.push(basis.get_transform(FRAC_PI_4, FRAC_1_SQRT_2));
        //     transforms.push(basis.get_transform(base_angle, 1.));
        // } 

        // // over half turn 
        // if angle > PI { 
        //     base_angle = PI;
        //     nurbs.knots.extend([base_angle, base_angle]);
        //     nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
        //     transforms.push(basis.get_transform(FRAC_PI_4*3., FRAC_1_SQRT_2)); 
        //     transforms.push(basis.get_transform(base_angle, 1.));
        // } 

        // // over three quarter turn  
        // if angle > FRAC_PI_2*3. { 
        //     base_angle = FRAC_PI_2*3.;
        //     nurbs.knots.extend([base_angle, base_angle]);
        //     nurbs.weights.extend([FRAC_1_SQRT_2, 1.]);
        //     transforms.push(basis.get_transform(FRAC_PI_4*5., FRAC_1_SQRT_2));
        //     transforms.push(basis.get_transform(base_angle, 1.));
        // }