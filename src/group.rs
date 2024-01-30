use super::{Model, DiscreteQuery, log};
//use super::mesh::{Mesh, get_trivec_with_offset};
use serde::{Deserialize, Serialize};
use glam::*;
use lyon::path::path::Builder;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Group::default")]
pub struct Group {
    pub parts:    Vec<Model>,
    //pub position: Box<Model>,
    pub axis:     Box<Model>,
    pub angle:    f32,
}

impl Group {
    pub fn add_parts_to_builder(&self, builder: &mut Builder) { 
        for part in &self.parts {
            match part {
                Model::Path(m)      => m.add_parts_to_builder(builder),
                Model::Group(m)     => m.add_parts_to_builder(builder),
                Model::Circle(m)    => m.add_self_to_builder(builder), 
                Model::Rectangle(m) => m.add_self_to_builder(builder),
                _ => (),
            };
        }
    }
    pub fn get_polyline(&self, query: &DiscreteQuery) -> Vec<f32> {
        let matrix = self.get_matrix(); // let matrix = Mat4::from_axis_angle(self.axis.get_vec3(), self.angle.to_degrees());
        let part = self.get_first_part(); 
        let polyline = match &part {
            Model::Group(m) => m.get_transformed_polyline(query, matrix),
            _ => get_transformed_vector(&part.get_polyline(query), matrix),
        };
        polyline 
    }
    fn get_transformed_polyline(&self, query: &DiscreteQuery, root_matrix: Mat4) -> Vec<f32> {
        let matrix = root_matrix * self.get_matrix(); // let matrix = root_matrix * Mat4::from_axis_angle(self.axis.get_vec3(), self.angle.to_degrees());
        let part = self.get_first_part(); 
        match &part {
            Model::Group(m) => m.get_transformed_polyline(query, matrix),
            _ => get_transformed_vector(&part.get_polyline(query), matrix),
        }
    }
    fn get_matrix(&self) -> Mat4 {
        let axis = self.axis.get_vec3();
        if axis.length() > 0. {
            Mat4::from_axis_angle(self.axis.get_vec3(), self.angle.to_degrees())
        }else{
            Mat4::IDENTITY
        }
    }
    fn get_first_part(&self) -> Model{
        self.parts.first().unwrap_or(&Model::Vector(vec![])).clone()
    }
}

impl Model {
    fn get_vec3(&self) -> Vec3 {
        match self {
            Model::Vector(m) => Vec3::from_slice(m),
            _ => Vec3::X,
        }
    }
}

fn get_transformed_vector(vector: &Vec<f32>, matrix: Mat4) -> Vec<f32> {
    vector.chunks(3).map(|v| {
        let vec4 = Vec4::new(v[0], v[1], v[2], 1.); //Vec4::from_slice(v);
        matrix.mul_vec4(vec4).to_array()
    }).flatten().collect()
}


// fn get_mat4_from_model(model: &Model) -> Mat4 {
//     match model {
//         Model::Vector(m) => point(m[0], m[1]),
//         _ => point(0., 0.),
//     }
// }