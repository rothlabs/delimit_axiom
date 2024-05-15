pub mod shapes;
pub mod groups;

mod basis3;
mod shader_parts;
mod shaders2;
mod shader_parts2;
mod shaders3;
mod shader_parts3;
mod traced;
mod flat;
mod hit3;
mod spread;

use std::f32::INFINITY;

use glam::*;
use crate::gpu::framebuffer::Framebuffer;
use crate::Shape;
//use hit2::hit2;
//use hit3::hit3;

// pub trait ToHit {
//     fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>);
// }

// impl ToHit for Vec<Shape> {
//     fn hit(self, pairs: &Vec<TestPair>) -> (Vec<HitPair>, Vec<OutPair>) {
//         //if self.high_rank() < 2 {
//             hit2(self, pairs)
//         //} else {
//         //    hit3(self, pairs)
//         //}
//     }
// }


#[derive(Clone, Debug)]
pub struct TestPair {
    pub i0: usize,
    pub i1: usize,
    pub reverse: bool,
}

#[derive(Clone, Default)]
pub struct Score {
    pub hits: Vec<Hit>,
    pub outs: Vec<Out>,
}

#[derive(Clone)]
pub struct HitPair {
    pub test: TestPair,
    pub shape: Shape,
    pub hits: (Hit, Hit),
}

pub fn hit_shape(shape: Shape) -> Hit {
    Hit {
        u:     0.,
        dot:   0.,
        shape: Some(shape),
        twin:  vec![],
    }
}

#[derive(Clone)]
pub struct Hit {
    pub u:     f32,
    pub dot:   f32,
    pub shape: Option<Shape>,
    pub twin:  Vec<usize>,
}

impl Hit {
    pub fn twined(&self, twin: Vec<usize>) -> Hit {
        let mut hit = self.clone();
        hit.twin = twin;
        hit
    }
}

#[derive(Clone)]
pub struct OutPair {
    pub test: TestPair,
    pub outs: (Out, Out),
    // pub dots: (f32, f32),
    // pub distance: f32,
}

#[derive(Clone)]
pub struct Out {
    pub dot: f32,
    pub distance: f32,
}

pub trait ClosetOut {
    fn closest(&self) -> &Out;
}

impl ClosetOut for Vec<Out> {
    fn closest(&self) -> &Out {
        let mut closest = &self[0];
        let mut distance = INFINITY;
        for out in self {
            if out.distance < distance {
                closest  = out;
                distance = out.distance;
            }
        }
        &closest
    }
}

struct HoneBuffer {
    io:       Framebuffer,
    palette0: Framebuffer,
    palette1: Framebuffer,
}



// #[derive(Clone, Default)]
// pub struct HitMiss3 {
//     pub hits:   Vec<Shape>,
//     pub misses: Vec<Miss>,
// }

// #[derive(Clone)]
// pub struct HitPair3 {
//     pub pair:   TestPair,
//     pub curve0: Shape,
//     pub curve1: Shape,
//     pub curve2: Shape,
// }



    // pub fn new(jobs: &Vec<Vec<Vec<Shape>>>) -> Self {
    //     let (starts, indexes) = cascade_group_job_indexes(jobs);
    //     CascadeGroupJob {
    //         pairs: cascade_group_job_pairs(&starts, jobs), 
    //         indexes,
    //     }
    // }




// pub fn job_indexes<T>(jobs: &Vec<Vec<Vec<T>>>) -> ([Vec<usize>; 2], Vec<(usize, usize, usize)>) {
//     let mut indexes = vec![];
//     let mut starts = [vec![], vec![]];
//     let mut job_start = 0;
//     for (ji, groups) in jobs.iter().enumerate() {
//         starts[0].push(job_start);
//         job_start += groups.len();
//         let mut group_start = 0;
//         for (gi, items) in groups.iter().enumerate() {
//             starts[1].push(group_start);
//             group_start += items.len();
//             for i in 0..items.len(){
//                 indexes.push((ji, gi, i));
//             }
//         }
//     }
//     (starts, indexes)
// }



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