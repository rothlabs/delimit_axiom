use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::{Model, Shape, Nurbs, get_shapes, get_vec3_or};
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
        let basis = Basis {
            axis,
            translation: Mat4::from_translation(center),
            reverse_translation: Mat4::from_translation(-center),
        };
        let mut knots = vec![0.; 3];
        let mut weights = vec![1.];
        let mut transforms = vec![];
        let mut base_angle = 0.;

        // over quarter turn 
        if self.angle > FRAC_PI_2 { 
            base_angle = FRAC_PI_2; 
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(basis.get_transform(FRAC_PI_4, FRAC_1_SQRT_2));
            transforms.push(basis.get_transform(base_angle, 1.));
        } 

        // over half turn 
        if self.angle > PI { 
            base_angle = PI;
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(basis.get_transform(FRAC_PI_4*3., FRAC_1_SQRT_2)); 
            transforms.push(basis.get_transform(base_angle, 1.));
        } 

        // over three quarter turn  
        if self.angle > FRAC_PI_2*3. { 
            base_angle = FRAC_PI_2*3.;
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(basis.get_transform(FRAC_PI_4*5., FRAC_1_SQRT_2));
            transforms.push(basis.get_transform(base_angle, 1.));
        }

        // final turn 
        let advance = (self.angle - base_angle) / 2.;
        knots.extend([self.angle, self.angle, self.angle]);
        weights.extend([advance.cos(), 1.]);
        transforms.push(basis.get_transform(base_angle + advance, advance.cos()));
        let end_mat4 = basis.get_transform(self.angle, 1.);
        //let end_mat4 = Mat4::from_axis_angle(axis, self.angle);


        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            let get_nurbs = || {
                let mut nurbs = Nurbs {
                    order: 3,
                    knots: knots.clone(),
                    weights: weights.clone(),
                    controls: vec![shape.clone()], 
                    boundaries: vec![],
                    reversed: false,
                };
                for &mat4 in &transforms {
                    nurbs.controls.push(shape.get_transformed(mat4)); 
                }
                nurbs.controls.push(shape.get_transformed(end_mat4)); 
                nurbs
            };
            shapes.push(shape.clone());
            match &shape {
                Shape::Point(_) => {
                    shapes.push(Shape::Curve(get_nurbs()));
                    shapes.push(shape.get_transformed(end_mat4));
                },
                Shape::Curve(_) => {
                    shapes.push(Shape::Facet(get_nurbs()));
                    shapes.push(shape.get_transformed(end_mat4));
                },
                Shape::Facet(nurbs) => {
                    shapes.push(Shape::Facet(nurbs.get_reversed_and_transformed(end_mat4)));
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
}

// TODO: fix skewing from diagonal axis 
impl Basis {
    fn get_transform(&self, angle: f32, weight: f32) -> Mat4 {
        self.translation 
        * Mat4::from_scale(Vec3::new( 
            (1./weight)*(1.-self.axis.dot(Vec3::X).abs()) + self.axis.dot(Vec3::X).abs(),
            (1./weight)*(1.-self.axis.dot(Vec3::Y).abs()) + self.axis.dot(Vec3::Y).abs(),
            (1./weight)*(1.-self.axis.dot(Vec3::Z).abs()) + self.axis.dot(Vec3::Z).abs(),
        ))
        * Mat4::from_axis_angle(self.axis, angle)
        * self.reverse_translation
    }
}

// // TODO: fix skewing from diagonal axis 
// fn get_transform(center: Vec3, axis: Vec3, angle: f32, weight: f32) -> Mat4 {
//     Mat4::from_translation(-center) 
//     * Mat4::from_scale(Vec3::new( 
//         (1./weight)*(1.-axis.abs().dot(Vec3::X)) + axis.abs().dot(Vec3::X),
//         (1./weight)*(1.-axis.abs().dot(Vec3::Y)) + axis.abs().dot(Vec3::Y),
//         (1./weight)*(1.-axis.abs().dot(Vec3::Z)) + axis.abs().dot(Vec3::Z),
//     )) 
//     * Mat4::from_axis_angle(axis, angle)
// }



// for shape in get_shapes(&self.parts) {
//     let get_nurbs = || {
//         let mut nurbs = Nurbs {
//             order: 3,
//             knots: knots.clone(),
//             weights: weights.clone(),
//             controls: vec![shape.clone()], 
//             boundaries: vec![],
//         };
//         for &mat4 in &transforms {
//             nurbs.controls.push(shape.get_transformed(mat4)); 
//         }
//         let end_shape = shape.get_transformed(end_mat4);
//         nurbs.controls.push(end_shape.clone()); 
//         //shapes.push(end_shape);
//         nurbs
//     };
//     match &shape {
//         Model::Point(_) => shapes.push(Model::Curve(get_nurbs())),
//         Model::Curve(_) => shapes.push(Model::Facet(get_nurbs())),
//         _ => (),
//     }
//     shapes.push(shape.clone());
// }