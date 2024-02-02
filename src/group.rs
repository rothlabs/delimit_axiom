use super::{Model, DiscreteQuery};
use lyon::geom::euclid::Transform3D;
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
    pub fn get_paths(&self) -> Vec<lyon::path::Path> { //builder: &mut Builder) 
        self.get_transformed_paths(Mat4::IDENTITY)
    }
    pub fn get_transformed_paths(&self, root_matrix: Mat4) -> Vec<lyon::path::Path> { //builder: &mut Builder) 
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
        //let axis = &self.axis.unwrap_or(Box::new(Model::Vector(vec![0.;3]))).get_vec3_or(Vec3::Z);
        let position = self.position.get_vec3_or(Vec3::ZERO);
        mat4 *= Mat4::from_translation(position);
        let axis = self.axis.get_vec3_or(Vec3::Z);
        mat4 *= Mat4::from_axis_angle(axis, self.angle);
        let scale = self.scale.get_vec3_or(Vec3::ONE);
        mat4 *= Mat4::from_scale(scale);
        mat4
    }
}

pub fn get_transformed_vector(vector: &Vec<f32>, matrix: Mat4) -> Vec<f32> {
    let mut result = vec![];
    for v in vector.chunks(3) {
        let vec4 = Vec4::new(v[0], v[1], v[2], 1.); //Vec4::from_slice(v);
        let array = matrix.mul_vec4(vec4).to_array();
        result.extend([array[0], array[1], array[2]]);
    }
    result
}




// for part in &self.parts {
        //     match part {
        //         Model::Path(m)      => m.add_parts_to_builder(builder),
        //         Model::Group(m)     => m.add_parts_to_builder(builder),
        //         Model::Circle(m)    => builder.extend_from_paths(&[m.get_path().transformed(&transform).as_slice()]),//m.add_self_to_builder(builder), 
        //         Model::Rectangle(m) => builder.extend_from_paths(&[m.get_path().transformed(&transform).as_slice()]),
        //         _ => (),
        //     };
        // }

// fn get_first_part(&self) -> Model{
    //     self.parts.first().unwrap_or(&Model::Vector(vec![])).clone()
    // }


// vector.chunks(3).map(|v| {
//     let vec4 = Vec4::new(v[0], v[1], v[2], 1.); //Vec4::from_slice(v);
//     matrix.mul_vec4(vec4).to_array()
// }).flatten().collect()




// fn get_mat4_from_model(model: &Model) -> Mat4 {
//     match model {
//         Model::Vector(m) => point(m[0], m[1]),
//         _ => point(0., 0.),
//     }
// }