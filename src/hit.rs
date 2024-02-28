pub mod hit2;
pub mod hit3;

#[derive(Clone)]
pub struct Miss {
    pub dot: f32,
    pub distance: f32,
}