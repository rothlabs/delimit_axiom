use crate::shape::*;
use super::{TestPair, Score};
use super::flat::HitTest;

pub trait HitTestShapes {
    fn hit(&self) -> Vec<Vec<Score>>;
}

impl HitTestShapes for Vec<Vec<Shape>> {
    fn hit(&self) -> Vec<Vec<Score>> {
        let (indices, starts) = indices_and_starts(self);
        let index = Index{indices,  pairs:pairs(self, &starts)};
        let shapes: Vec<Shape> = self.clone().into_iter().flatten().collect();
        let (hit_pairs, out_pairs) = shapes.hit(&index.pairs);
        let mut hits:   Vec<Vec<Score>> = vec![vec![]; self.len()];
        for (ji, shapes) in self.iter().enumerate() {
            hits[ji].extend(vec![Score::default(); shapes.len()]);
        }
        for pair in hit_pairs {
            let (j, i0, i1) = index.at(&pair.test);
            hits[j][i0].hits.push(pair.hits.0);
            hits[j][i1].hits.push(pair.hits.1);
        }
        for pair in out_pairs {
            let (j, i0, i1) = index.at(&pair.test);
            hits[j][i0].outs.push(pair.outs.0);
            hits[j][i1].outs.push(pair.outs.1);
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
        let (j, i0) = self.indices[pair.i0];
        let (_, i1) = self.indices[pair.i1];
        (j, i0, i1)
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

pub fn pairs(jobs: &Vec<Vec<Shape>>, starts: &Vec<usize>) -> Vec<TestPair> {
    let mut pairs = vec![];
    for (ji, shapes) in jobs.iter().enumerate() {
        for i0 in 0..shapes.len() {
            for i1 in i0+1..shapes.len() {
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