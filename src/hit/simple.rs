use crate::shape::*;
use super::{ToHit, TestPair, HitMiss, Miss};

pub trait HitTest {
    fn hit(&self) -> Vec<Vec<HitMiss>>;
}

impl HitTest for Vec<Vec<Shape>> {
    fn hit(&self) -> Vec<Vec<HitMiss>> {
        let (indices, starts) = indices_and_starts(self);
        let index = Index{indices,  pairs:job_pairs(self, &starts)};
        let shapes: Vec<Shape> = self.clone().into_iter().flatten().collect();
        let (hits2, misses) = shapes.hit(&index.pairs);
        let mut hits:   Vec<Vec<HitMiss>> = vec![vec![]; self.len()];
        for (ji, shapes) in self.iter().enumerate() {
            hits[ji].extend(vec![HitMiss::default(); shapes.len()]);
        }
        for hit in &hits2 {
            let (ji, i0, i1) = index.at(&hit.pair);
            hits[ji][i0].hits.push(hit.hits.0.clone());
            hits[ji][i1].hits.push(hit.hits.1.clone());
        }
        for miss in &misses {
            let (ji, i0, i1) = index.at(&miss.pair);
            hits[ji][i0].misses.push(Miss{dot:miss.dots.0, distance:miss.distance});
            hits[ji][i1].misses.push(Miss{dot:miss.dots.1, distance:miss.distance});
        }
        hits
    }
}

pub struct Index {
    pub pairs: Vec<TestPair>,
    indices:   Vec<(usize, usize)>,
}

impl Index {
    pub fn at(&self, pair: &TestPair) -> (usize, usize, usize) {
        let (ji, i0) = self.indices[pair.i0];
        let (_,  i1) = self.indices[pair.i1];
        (ji, i0, i1)
    }
}

fn indices_and_starts(jobs: &Vec<Vec<Shape>>) -> (Vec<(usize, usize)>, Vec<usize>) {
    let mut indices = vec![];
    let mut starts  = vec![];
    let mut job_start = 0;
    for (ji, shapes) in jobs.iter().enumerate() {
        starts.push(job_start);
        job_start += shapes.len();
        for i in 0..shapes.len(){
            indices.push((ji, i));
        }
    }
    (indices, starts)
}

pub fn job_pairs(jobs: &Vec<Vec<Shape>>, starts: &Vec<usize>) -> Vec<TestPair> {
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