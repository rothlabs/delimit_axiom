mod basis3;
mod shader_parts;
mod shaders2;
mod shader_parts2;
mod shaders3;
mod shader_parts3;
mod traced;
pub mod hit2;
pub mod hit3;

use glam::*;
use crate::{gpu::framebuffer::Framebuffer, Shape};


pub struct HitJob {
    pub pairs: Vec<TestPair>,
    indexes:   Vec<(usize, usize)>,
}

impl HitJob {
    pub fn new(jobs: &Vec<Vec<Shape>>) -> Self {
        let (starts, indexes) = job_indexes(jobs);
        HitJob {
            pairs: job_pairs(&starts, jobs), 
            indexes,
        }
    }
    pub fn index(&self, pair: &TestPair) -> (usize, usize, usize) {
        let (ji, i0) = self.indexes[pair.i0];
        let (_,  i1) = self.indexes[pair.i1];
        (ji, i0, i1)
    }
}

fn job_indexes(jobs: &Vec<Vec<Shape>>) -> (Vec<usize>, Vec<(usize, usize)>) {
    let mut indexes = vec![];
    let mut starts  = vec![];
    let mut job_start = 0;
    for (ji, shapes) in jobs.iter().enumerate() {
        starts.push(job_start);
        job_start += shapes.len();
        for i in 0..shapes.len(){
            indexes.push((ji, i));
        }
    }
    (starts, indexes)
}

pub fn job_pairs(starts: &Vec<usize>, jobs: &Vec<Vec<Shape>>) -> Vec<TestPair> {
    let mut pairs = vec![];
    for (ji, shapes) in jobs.iter().enumerate() {
        for i0 in 0..shapes.len() {
            for i1 in i0+1..shapes.len() {
                //if i0 == i1 {continue}
                pairs.push(TestPair{
                    i0: starts[ji] + i0, 
                    i1: starts[ji] + i1,
                    reverse: false,
                });
            }
        }
    }
    pairs
}


pub struct CascadeGroupJob {
    pub pairs: Vec<TestPair>,
    indexes:   Vec<(usize, usize, usize)>,
}

impl CascadeGroupJob {
    pub fn new(jobs: &Vec<Vec<Vec<Shape>>>) -> Self {
        let (starts, indexes) = cascade_group_job_indexes(jobs);
        CascadeGroupJob {
            pairs: cascade_group_job_pairs(&starts, jobs), 
            indexes,
        }
    }
    pub fn index(&self, pair: &TestPair) -> (usize, usize, usize, usize, usize) {
        let (_,  g0, i0) = self.indexes[pair.i0];
        let (ji, g1, i1) = self.indexes[pair.i1];
        (ji, g0, i0, g1, i1)
    }
}

fn cascade_group_job_indexes<T>(jobs: &Vec<Vec<Vec<T>>>) -> ([Vec<usize>; 2], Vec<(usize, usize, usize)>) {
    let mut indexes = vec![];
    let mut starts = [vec![], vec![]];
    let mut job_start = 0;
    for (ji, groups) in jobs.iter().enumerate() {
        starts[0].push(job_start);
        job_start += groups.len();
        let mut group_start = 0;
        for (gi, items) in groups.iter().enumerate() {
            starts[1].push(group_start);
            group_start += items.len();
            for i in 0..items.len(){
                indexes.push((ji, gi, i));
            }
        }
    }
    (starts, indexes)
}

// TODO: limit by shape rank
pub fn cascade_group_job_pairs(starts: &[Vec<usize>; 2], jobs: &Vec<Vec<Vec<Shape>>>) -> Vec<TestPair> {
    let mut pairs = vec![];
    for (ji, groups) in jobs.iter().enumerate() {
        for g1 in 1..groups.len(){
            for g0 in 0..g1 {
                for c0 in 0..groups[g0].len(){
                    for c1 in 0..groups[g1].len(){
                        //if groups[g0][c0].rank == 1 && groups[g1][c1].rank == 1 {
                        let mut reverse = false;
                        if groups[g0][c0].space.sign != groups[g1][c1].space.sign {
                            reverse = true; 
                        }
                            pairs.push(TestPair{
                                i0: starts[0][ji] + starts[1][g0] + c0, 
                                i1: starts[0][ji] + starts[1][g1] + c1,
                                reverse,
                            });
                        //}
                    }  
                }   
            }
        }
    }
    pairs
}





#[derive(Clone, Debug)]
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

#[derive(Clone, Default)]
pub struct HitMiss3 {
    pub hits:   Vec<Shape>,
    pub misses: Vec<Miss>,
}

#[derive(Clone)]
pub struct HitPair3 {
    pub pair:   TestPair,
    pub curve0: Shape,
    pub curve1: Shape,
    pub curve2: Shape,
}

struct HoneBuffer {
    io:       Framebuffer,
    palette0: Framebuffer,
    palette1: Framebuffer,
}










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