use glam::Vec3;

pub mod hit2;
pub mod hit3;
mod basis3;
mod shader;
mod shader_parts;
mod traced;

#[derive(Clone)]
pub struct IndexPair {
    pub g0: usize,
    pub g1: usize,
    pub i0: usize,
    pub i1: usize,
}

#[derive(Clone)]
pub struct Miss {
    pub distance: f32,
    pub dot: f32,
}

#[derive(Clone)]
pub struct MissPair {
    pub index: IndexPair,
    pub distance: f32,
    pub dot0: f32,
    pub dot1: f32,
}