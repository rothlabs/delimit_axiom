use glam::Vec3;

pub mod hit2;
pub mod hit3;
mod basis3;
mod shader;
mod shader_parts;
mod traced;

// #[derive(Clone, Default, Serialize, Deserialize)]
// #[serde(default = "FacetHit::default")]
// pub struct FacetHit {
//     g0: usize,
//     g1: usize,
//     f0: usize,
//     f1: usize,
//     uv0: [usize; 2],
// }

#[derive(Clone)]
pub struct Miss {
    pub dot: f32,
    pub distance: f32,
    pub point: Vec3,
}