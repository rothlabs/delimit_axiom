mod basis3;
mod shader_parts;
mod shaders2;
mod shader_parts2;
mod shaders3;
mod shader_parts3;
mod traced;
pub mod hit2;
pub mod hit2_gpu;
pub mod hit3;

use glam::*;
use crate::{gpu::framebuffer::Framebuffer, CurveShape};


struct HoneBuffer {
    io:       Framebuffer,
    palette0: Framebuffer,
    palette1: Framebuffer,
}

#[derive(Clone)]
pub struct MissPair {
    pub i0: usize,
    pub i1: usize,
    pub dot0: f32,
    pub dot1: f32,
    pub distance: f32,
}

#[derive(Clone)]
pub struct Miss {
    pub dot: f32,
    pub distance: f32,
}

#[derive(Clone)]
pub struct Hit2 {
    pub i0: usize,
    pub i1: usize,
    pub u0: f32,
    pub u1: f32,
    pub dot0: f32,
    pub dot1: f32,
    pub point: Vec3,
}

#[derive(Clone)]
pub struct Hit3 {
    pub i0: usize,
    pub i1: usize,
    pub curve0: CurveShape,
    pub curve1: CurveShape,
    pub curve2: CurveShape,
}

#[derive(Clone)]
pub struct TestPair3 {
    pub i0: usize,
    pub i1: usize,
    pub reverse: bool,
}

// #[derive(Clone)]
// pub struct MissPair {
//     pub index: TestPair3,
//     pub distance: f32,
//     pub dot0: f32,
//     pub dot1: f32,
// }


// #[derive(Clone)]
// pub struct CurveHit {
//     pub u: f32,
//     pub dot: f32,
// }

// #[derive(Clone)]
// pub struct Hit2 {
//     pub hit: (CurveHit, CurveHit),
//     pub center: Vec3,
// }