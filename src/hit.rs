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

pub fn job_indexes<T>(jobs: &Vec<Vec<Vec<T>>>) -> ([Vec<usize>; 2], Vec<(usize, usize, usize)>) {
    let mut indexes = vec![];
    let mut starts = [vec![], vec![]];
    let mut job_start = 0;
    for (ji, job) in jobs.iter().enumerate() {
        starts[0].push(job_start);
        job_start += job.len();
        let mut group_start = 0;
        for (gi, group) in job.iter().enumerate() {
            starts[1].push(group_start);
            group_start += group.len();
            for ci in 0..group.len(){
                indexes.push((ji, gi, ci));
            }
        }
    }
    (starts, indexes)
}

#[derive(Clone)]
pub struct TestPair {
    pub i0: usize,
    pub i1: usize,
    pub reverse: bool,
}

#[derive(Clone)]
pub struct MissPair {
    pub pair: TestPair,
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
pub struct HitPair2 {
    pub pair: TestPair,
    pub u0: f32,
    pub u1: f32,
    pub dot0: f32,
    pub dot1: f32,
    pub point: Vec3,
}

#[derive(Clone)]
pub struct Hit2 {
    pub u: f32,
    pub dot: f32,
}

#[derive(Clone, Default)]
pub struct HitMiss2 {
    pub hits:   Vec<Hit2>,
    pub misses: Vec<Miss>,
}

#[derive(Clone)]
pub struct HitPair3 {
    pub pair:   TestPair,
    pub curve0: CurveShape,
    pub curve1: CurveShape,
    pub curve2: CurveShape,
}

struct HoneBuffer {
    io:       Framebuffer,
    palette0: Framebuffer,
    palette1: Framebuffer,
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