use crate::shape::*;
use super::{ToHit, TestPair, HitMiss, Miss};

pub trait HitTest {
    fn hit(&self) -> Vec<Vec<Vec<HitMiss>>>;
}

impl HitTest for Vec<Vec<Vec<Shape>>> {
    fn hit(&self) -> Vec<Vec<Vec<HitMiss>>> {
        let (indices, starts) = indices_and_starts(self);
        let index = Index {
            pairs: pairs(self, &starts), 
            indices,
        };
        let shapes: Vec<Shape> = self.clone().into_iter().flatten().flatten().collect();
        let (hit_pairs, miss_pairs) = shapes.hit(&index.pairs);
        let mut hits: Vec<Vec<Vec<HitMiss>>> = vec![vec![]; self.len()];
        for (ji, groups) in self.iter().enumerate() {
            for gi in 0..groups.len() {
                hits[ji].push(vec![HitMiss::default(); groups[gi].len()]);
            }
        }
        for hit in &hit_pairs {
            let (ji, g0, i0, g1, i1) = index.at(&hit.pair);
            hits[ji][g0][i0].hits.push(hit.hits.0.twined(vec![g1, i1]));
            hits[ji][g1][i1].hits.push(hit.hits.1.twined(vec![g0, i0]));
        }
        for miss in &miss_pairs {
            let (ji, g0, i0, g1, i1) = index.at(&miss.pair);
            hits[ji][g0][i0].misses.push(Miss{dot:miss.dots.0, distance:miss.distance});
            hits[ji][g1][i1].misses.push(Miss{dot:miss.dots.1, distance:miss.distance});
        }
        hits
    }
}

pub struct Index {
    pub pairs: Vec<TestPair>,
    indices:   Vec<(usize, usize, usize)>,
}

impl Index {
    pub fn at(&self, pair: &TestPair) -> (usize, usize, usize, usize, usize) {
        let (_,  g0, i0) = self.indices[pair.i0];
        let (ji, g1, i1) = self.indices[pair.i1];
        (ji, g0, i0, g1, i1)
    }
}

fn indices_and_starts<T>(jobs: &Vec<Vec<Vec<T>>>) -> (Vec<(usize, usize, usize)>, [Vec<usize>; 2]) {
    let mut starts = [vec![], vec![]];
    let mut indices = vec![];
    let mut job_start = 0;
    for (ji, groups) in jobs.iter().enumerate() {
        starts[0].push(job_start);
        job_start += groups.len();
        let mut group_start = 0;
        for (gi, items) in groups.iter().enumerate() {
            starts[1].push(group_start);
            group_start += items.len();
            for i in 0..items.len(){
                indices.push((ji, gi, i));
            }
        }
    }
    (indices, starts)
}

// TODO: limit by shape rank
pub fn pairs(jobs: &Vec<Vec<Vec<Shape>>>, starts: &[Vec<usize>; 2]) -> Vec<TestPair> {
    let mut pairs = vec![];
    for (ji, groups) in jobs.iter().enumerate() {
        for g1 in 1..groups.len(){
            for g0 in 0..g1 {
                for c0 in 0..groups[g0].len(){
                    for c1 in 0..groups[g1].len(){
                        //if groups[g0][c0].rank == 1 && groups[g1][c1].rank == 1 {
                        // let mut reverse = false;
                        // if groups[g0][c0].basis.sign != groups[g1][c1].basis.sign {
                        //     reverse = true; 
                        // }
                            pairs.push(TestPair{
                                i0: starts[0][ji] + starts[1][g0] + c0, 
                                i1: starts[0][ji] + starts[1][g1] + c1,
                                reverse: true,
                            });
                        //}
                    }  
                }   
            }
        }
    }
    pairs
}