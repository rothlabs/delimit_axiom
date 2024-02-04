use crate::{get_path_from_parts, Group, Nurbs};
use super::{Model, DiscreteQuery, vector::*, log};
use super::mesh::{Mesh, get_trivec_with_offset};
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
    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh {
        let axis = self.axis.get_vec3_or(Vec3::X).normalize();
        let weight = (self.angle/2.).cos();
        let scale = Mat4::from_scale(Vec3::new(
            (1./weight)*(1.-axis.abs().dot(Vec3::X)) + axis.abs().dot(Vec3::X),
            (1./weight)*(1.-axis.abs().dot(Vec3::Y)) + axis.abs().dot(Vec3::Y),
            (1./weight)*(1.-axis.abs().dot(Vec3::Z)) + axis.abs().dot(Vec3::Z),
        ));
        let half_rotation = Mat4::from_axis_angle(axis, self.angle / 2.);
        let rotation = Mat4::from_axis_angle(axis, self.angle);
        let mut surfaces = vec![];
        for part in &self.parts {
            for nurbs in part.get_nurbs_vec() {
                let mut surface = Nurbs::default();
                surface.controls.push(Model::Nurbs(nurbs.clone()));
                surface.controls.push(Model::Nurbs(nurbs.get_transformed(scale).get_transformed(half_rotation)));
                surface.controls.push(Model::Nurbs(nurbs.get_transformed(rotation)));
                surface.order = 3;
                surface.knots = vec![0., 0., 0., self.angle, self.angle, self.angle];
                surface.weights = vec![1., weight, 1.];
                surfaces.push(surface);
            }
        }
        //let mut vector: Vec<f32> = vec![];
        //let mut trivec: Vec<usize> = vec![];

        surfaces[0].get_mesh(query)
    }
}