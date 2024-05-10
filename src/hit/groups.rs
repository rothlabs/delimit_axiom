use crate::shape::*;
use super::{ToHit, TestPair, Score};

pub trait HitTest {
    fn hit(&self) -> Vec<[Vec<Score>; 2]>;
}

impl HitTest for Vec<[Vec<Shape>; 2]> {
    fn hit(&self) -> Vec<[Vec<Score>; 2]> {
        let (indices, starts) = indices_and_starts(self);
        let index = Index {indices, pairs:pairs(self, &starts)};
        let shapes: Vec<Shape> = self.clone().into_iter().flatten().flatten().collect();
        let (hit_pairs, out_pairs) = shapes.hit(&index.pairs);
        let mut score: Vec<[Vec<Score>; 2]> = vec![];
        for groups in self {
            score.push([
                vec![Score::default(); groups[0].len()],
                vec![Score::default(); groups[1].len()]
            ]);
        }
        for pair in hit_pairs {
            let (j, g0, i0, g1, i1) = index.at(&pair.test);
            score[j][g0][i0].hits.push(pair.hits.0);   // .twined(vec![g1, i1])
            score[j][g1][i1].hits.push(pair.hits.1);   // .twined(vec![g0, i0])
        }
        for pair in out_pairs {
            let (j, g0, i0, g1, i1) = index.at(&pair.test);
            score[j][g0][i0].outs.push(pair.outs.0);
            score[j][g1][i1].outs.push(pair.outs.1);
        }
        score
    }
}

pub struct Index {
    pub pairs: Vec<TestPair>,
    indices:   Vec<(usize, usize, usize)>,
}

impl Index {
    pub fn at(&self, pair: &TestPair) -> (usize, usize, usize, usize, usize) {
        let (j, g0, i0) = self.indices[pair.i0];
        let (_, g1, i1) = self.indices[pair.i1];
        (j, g0, i0, g1, i1)
    }
}

fn indices_and_starts<T>(jobs: &Vec<[Vec<T>; 2]>) -> (Vec<(usize, usize, usize)>, Vec<usize>) {
    let mut starts =  vec![];
    let mut indices = vec![];
    let mut job_start = 0;
    for (ji, groups) in jobs.iter().enumerate() {
        starts.push(job_start);
        for (gi, shapes) in groups.iter().enumerate() {
            job_start += shapes.len();
            for i in 0..shapes.len(){
                indices.push((ji, gi, i));
            }
        }
    }
    (indices, starts)
}

// TODO: limit by shape rank
pub fn pairs(jobs: &Vec<[Vec<Shape>; 2]>, starts: &Vec<usize>) -> Vec<TestPair> {
    let mut pairs = vec![];
    for (ji, groups) in jobs.iter().enumerate() {
        for i0 in 0..groups[0].len(){
            for i1 in 0..groups[1].len(){
                pairs.push(TestPair{
                    i0: starts[ji] + i0, 
                    i1: starts[ji] + groups[0].len() + i1,
                    reverse: false,
                });
            }  
        }   
    }
    pairs
}


// pub fn pairs(jobs: &Vec<Vec<Vec<Shape>>>, starts: &[Vec<usize>; 2]) -> Vec<TestPair> {
//     let mut pairs = vec![];
//     for (ji, groups) in jobs.iter().enumerate() {
//         for g1 in 1..groups.len(){
//             for g0 in 0..g1 {
//                 for c0 in 0..groups[g0].len(){
//                     for c1 in 0..groups[g1].len(){
//                         //if groups[g0][c0].rank == 1 && groups[g1][c1].rank == 1 {
//                         // let mut reverse = false;
//                         // if groups[g0][c0].basis.sign != groups[g1][c1].basis.sign {
//                         //     reverse = true; 
//                         // }
//                             pairs.push(TestPair{
//                                 i0: starts[0][ji] + starts[1][g0] + c0, 
//                                 i1: starts[0][ji] + starts[1][g1] + c1,
//                                 reverse: true,
//                             });
//                         //}
//                     }  
//                 }   
//             }
//         }
//     }
//     pairs
// }