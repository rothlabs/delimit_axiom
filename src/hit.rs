use glam::Vec3;

pub mod hit2;
pub mod hit3;

#[derive(Clone)]
pub struct Miss {
    pub dot: f32,
    pub distance: f32,
    pub point: Vec3,
}