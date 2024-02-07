use std::f32::consts::{PI, FRAC_PI_2, FRAC_PI_4, FRAC_1_SQRT_2};
use crate::{get_shapes, Nurbs};
use super::{Model, DiscreteQuery, log};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Revolve::default")]
pub struct Revolve {
    pub parts:  Vec<Model>,
    pub axis:   Box<Model>,
    pub angle:  f32,
}


impl Revolve {
    pub fn get_shapes(&self) -> Vec<Model> { // , query: &DiscreteQuery
        let axis = self.axis.get_vec3_or(Vec3::Z).normalize();
        let mut knots = vec![0.; 3];
        let mut weights = vec![1.];
        let mut transforms = vec![];
        let mut base_angle = 0.;

        // quarter turn controls
        if self.angle > FRAC_PI_2 { 
            base_angle = FRAC_PI_2; 
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(get_transform(axis, FRAC_PI_4, FRAC_1_SQRT_2));
            transforms.push(get_transform(axis, base_angle, 1.));
        } 

        // half turn controls
        if self.angle > PI { 
            base_angle = PI;
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(get_transform(axis, FRAC_PI_4*3., FRAC_1_SQRT_2));
            transforms.push(get_transform(axis, base_angle, 1.));
        } 

        // three quarter turn controls 
        if self.angle > FRAC_PI_2*3. { 
            base_angle = FRAC_PI_2*3.;
            knots.extend([base_angle, base_angle]);
            weights.extend([FRAC_1_SQRT_2, 1.]);
            transforms.push(get_transform(axis, FRAC_PI_4*5., FRAC_1_SQRT_2));
            transforms.push(get_transform(axis, base_angle, 1.));
        }

        // add final controls 
        let advance = (self.angle - base_angle) / 2.;
        knots.extend([self.angle, self.angle, self.angle]);
        weights.extend([advance.cos(), 1.]);
        transforms.push(get_transform(axis, base_angle + advance, advance.cos()));
        transforms.push(Mat4::from_axis_angle(axis, self.angle));

        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            let get_nurbs = || {
                let mut nurbs = Nurbs {
                    order: 3,
                    knots: knots.clone(),
                    weights: weights.clone(),
                    controls: vec![shape.clone()], 
                    boundaries: vec![],
                };
                for &mat4 in &transforms {
                    nurbs.controls.push(shape.get_transformed(mat4)); 
                }
                nurbs
            };
            match &shape {
                Model::Point(_) => shapes.push(Model::Curve(get_nurbs())),
                Model::Curve(_) => shapes.push(Model::Facet(get_nurbs())),
                _ => (),
            }
        }
        shapes 
    }
}

// TODO: fix skewing from diagonal axis 
fn get_transform(axis: Vec3, angle: f32, weight: f32) -> Mat4 {
    Mat4::from_scale(Vec3::new( 
        (1./weight)*(1.-axis.abs().dot(Vec3::X)) + axis.abs().dot(Vec3::X),
        (1./weight)*(1.-axis.abs().dot(Vec3::Y)) + axis.abs().dot(Vec3::Y),
        (1./weight)*(1.-axis.abs().dot(Vec3::Z)) + axis.abs().dot(Vec3::Z),
    )) * Mat4::from_axis_angle(axis, angle)
}