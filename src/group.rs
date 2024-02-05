use crate::Nurbs;

use super::{Model, DiscreteQuery, vector::*, log};
use lyon::{geom::euclid::Transform3D, path::traits::PathIterator};
//use lyon::geom::euclid::Transform3D;
//use super::mesh::{Mesh, get_trivec_with_offset};
use serde::{Deserialize, Serialize};
use glam::*;
//use lyon::path::path::Builder;



#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Group::default")]
pub struct Group {
    pub parts:    Vec<Model>,
    pub position: Box<Model>,
    pub axis:     Box<Model>,
    pub angle:    f32,
    pub scale:    Box<Model>,
}

impl Group {
    pub fn get_controls(&self) -> Vec<Nurbs> {
        self.get_transformed_nurbs(Mat4::IDENTITY)
    }
    fn get_transformed_nurbs(&self, root_matrix: Mat4) -> Vec<Nurbs> {
        let mat4 = root_matrix * self.get_matrix();
        let mut nurbs = vec![];
        for part in &self.parts {
            match &part {
                Model::Group(m) => nurbs.extend(m.get_transformed_nurbs(mat4)),
                Model::Nurbs(m) => nurbs.push(m.get_transformed(mat4)),
                _ => ()
            }
        }
        nurbs
    }
    pub fn get_paths(&self) -> Vec<lyon::path::Path> { //builder: &mut Builder) 
        self.get_transformed_paths(Mat4::IDENTITY)
    }
    fn get_transformed_paths(&self, root_matrix: Mat4) -> Vec<lyon::path::Path> { //builder: &mut Builder) 
        let mat4 = root_matrix * self.get_matrix();
        let mat_array = mat4.to_cols_array();
        let transform = Transform3D::from_array(mat_array).to_2d();
        let mut paths = vec![];
        for part in &self.parts {
            match &part {
                Model::Group(m) => paths.extend(m.get_transformed_paths(mat4)),
                _ => paths.push(part.get_path().transformed(&transform)),
            }
        }
        paths // std::slice::from_ref(&
    }
    pub fn get_polylines(&self, query: &DiscreteQuery) -> Vec<Vec<f32>> {
        self.get_transformed_polylines(query, Mat4::IDENTITY)
    }
    fn get_transformed_polylines(&self, query: &DiscreteQuery, root_matrix: Mat4) -> Vec<Vec<f32>> {
        let matrix = root_matrix * self.get_matrix();
        let mut polylines = vec![];
        for part in &self.parts {
            match &part {
                Model::Group(m) => polylines.extend(m.get_transformed_polylines(query, matrix)),
                _ => polylines.push(get_transformed_vector(&part.get_polyline(query), matrix)),
            }
        }
        polylines
    }
    fn get_matrix(&self) -> Mat4 {
        let mut mat4 = Mat4::IDENTITY;
        let position = self.position.get_vec3_or(Vec3::ZERO);
        mat4 *= Mat4::from_translation(position);
        let axis = self.axis.get_vec3_or(Vec3::Z);
        mat4 *= Mat4::from_axis_angle(axis, self.angle);
        let scale = self.scale.get_vec3_or(Vec3::ONE);
        mat4 *= Mat4::from_scale(scale);
        mat4
    }
}


// pub fn get_vector_at_t(&self, t: f32) -> Vec<f32> {
    //     let mut builder = lyon::path::Path::builder();
    //     for path in &self.get_paths() {
    //         builder.extend_from_paths(&[path.as_slice()]);
    //     }
    //     let path = builder.build();
    //     path.into_iter().flattened(0.1).count()

    //     //get_vector_at_t(&get_path_from_parts(&self.parts), t) 
    //     // TODO: get from nurbs curve instead of paths if present
    //     //log("group get_vector_at_t");
    //     // for path in &self.get_paths() {
    //     //     return get_vector_at_t(&path, t);
    //     //     // let [x, y] = get_vector_at_t(&path, t).to_array();
    //     //     // return vec![x, y, 0.];
    //     // }
    //     // vec![0.; 3]
    // }