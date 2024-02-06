use super::{Model, Parameter};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Slice::default")]
pub struct Slice {
    pub models: Vec<Model>,
    pub t: Parameter,
}

impl Slice {
    pub fn get_polyline(&self, count: usize) -> Vec<f32> {
        match &self.models[0] {
            Model::Facet(nurbs) => nurbs.get_polyline_at_t(&self.t, count),
            _ => vec![0.; 6],
        }
    }
}

// pub fn get_polyline_at_u(&self, count: usize) -> Vec<f32> {
//     match &self.models[0] {
//         Model::Nurbs(nurbs) => nurbs.get_polyline_at_u(self.t, count),
//         _ => vec![0.; 6],
//     }
// }
// pub fn get_polyline_at_v(&self, count: usize) -> Vec<f32> {
//     match &self.models[0] {
//         Model::Nurbs(nurbs) => nurbs.get_polyline_at_v(self.t, count),
//         _ => vec![0.; 6],
//     }
// }