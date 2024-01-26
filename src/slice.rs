use super::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Slice::default")]
pub struct Slice {
    pub models: Vec<Model>,
    pub t: f32,
}

impl Slice {
    pub fn get_polyline_at_u(&self, count: usize) -> Vec<f32> {
        match &self.models[0] {
            Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_u(self.t, count),
            _ => vec![0.; 6],
        }
    }
    pub fn get_polyline_at_v(&self, count: usize) -> Vec<f32> {
        match &self.models[0] {
            Model::Nurbs(nurbs) => nurbs.get_valid().get_polyline_at_v(self.t, count),
            _ => vec![0.; 6],
        }
    }
}