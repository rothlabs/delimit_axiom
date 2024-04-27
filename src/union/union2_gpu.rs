use crate::{log, CurveHit, CurveShape, HitTester2, Shape, Spatial3, UnionBatch};
use crate::hit::{hit2_gpu::HitTest2, Miss, HitMiss2, Hit2};
use glam::*;

pub trait UnionBatch2 {
    fn union(self) -> Vec<Vec<CurveShape>>;
}

impl UnionBatch2 for Vec<Vec<Vec<CurveShape>>> { // jobs, groups, curves
    fn union(self) -> Vec<Vec<CurveShape>> { 
        let batch = UnionBatch::new(&self);
        let curves: Vec<CurveShape> = self.clone().into_iter().flatten().flatten().collect();
        let (flat_hits, misses) = curves.hit(&batch.pairs);
        let mut hits:   Vec<[Vec<HitMiss2>; 2]> = vec![[vec![], vec![]]; self.len()];
        for (ji, groups) in self.iter().enumerate() {
            hits[ji][0].extend(vec![HitMiss2::default(); groups[0].len()]);
            hits[ji][1].extend(vec![HitMiss2::default(); groups[1].len()]);
        }
        for hit in &flat_hits {
            let (ji, g0, i0, g1, i1) = batch.index(&hit.pair);
            hits[ji][g0][i0].hits.push(Hit2{u:hit.u0, dot:hit.dot0});
            hits[ji][g1][i1].hits.push(Hit2{u:hit.u1, dot:hit.dot1});
        }
        for miss in &misses {
            let (ji, g0, i0, g1, i1) = batch.index(&miss.pair);
            hits[ji][g0][i0].misses.push(Miss{dot:miss.dot0, distance:miss.distance});
            hits[ji][g1][i1].misses.push(Miss{dot:miss.dot1, distance:miss.distance});
        }
        let mut results = vec![];
        for (ji, groups) in self.iter().enumerate() {
            let curves = UnionBasis2::new(groups[0].clone(), groups[1].clone(), hits[ji].clone());
            results.push(curves);
        }
        results
    }
}


// impl Union2 for Vec<Vec<CurveShape>> { // job, group, curve
//     fn union(self) { 
//         let batch = UnionBatch::new(&self);
//         let curves: Vec<CurveShape> = self.clone().into_iter().flatten().flatten().collect();
//         let (hits, misses) = curves.hit(&batch.pairs);
//         for hit in &hits {
//             let (ji, g0, i0, g1, i1) = batch.index(&hit.pair);
            
//         }
//     }
// }


pub struct UnionBasis2 {
    //pub tester: HitTester2,
    pub groups: [Vec<CurveShape>; 2],
    //pub hits:   [Vec<Vec<CurveHit>>; 2], 
    //pub miss:   [Vec<Vec<Miss>>; 2], 
    pub hits:   [Vec<HitMiss2>; 2], 
    pub curves: Vec<CurveShape>,
    pub shapes: Vec<Shape>,
}

impl UnionBasis2 { 
    pub fn new(curves0: Vec<CurveShape>, curves1: Vec<CurveShape>, hits:[Vec<HitMiss2>; 2]) -> Vec<CurveShape> {
        UnionBasis2 {
            // tester: HitTester2 {
            //     curves: (CurveShape::default(), CurveShape::default()),
            //     spatial: Spatial3::new(), 
            //     points:  vec![],
            // },
            hits,
            //hits: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
            //miss: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
            groups: [curves0, curves1],
            curves: vec![],
            shapes: vec![],
        }.build()
    }

    pub fn build(&mut self) -> Vec<CurveShape> {
        for g in 0..2 {
            for i in 0..self.groups[g].len() {
                if self.hits[g][i].hits.is_empty() {
                    self.hits[g][i].misses.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                    if self.hits[g][i].misses.is_empty() || self.hits[g][i].misses[0].dot * self.groups[g][i].nurbs.sign > 0. { 
                        self.curves.push(self.groups[g][i].clone());   
                    }
                }else{
                    self.hits[g][i].hits.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
                    self.add_bounded_curves(g, i);   
                }
            }
        }
        for curve in &mut self.curves {
            if curve.nurbs.sign < 0. {curve.reverse().negate();}
        }
        self.curves.clone()
    }

    fn add_bounded_curves(&mut self, g: usize, i: usize) {
        let mut curve = self.groups[g][i].clone();
        let min_basis = curve.min;
        for curve_hit in self.hits[g][i].hits.iter() {
            if curve_hit.dot * curve.nurbs.sign < 0. {
                curve.set_min(curve_hit.u);
            }else{
                let range = curve_hit.u - min_basis;
                if range < 0.0001 {
                    console_log!("union2 range: {}", range);
                }
                curve.set_max(min_basis, curve_hit.u);
                self.curves.push(curve);
                curve = self.groups[g][i].clone();
            }
        }
        if self.hits[g][i].hits.last().expect("There should be one or more hits.").dot * curve.nurbs.sign < 0. {
            self.curves.push(curve);
        }
    }
}



// let mut results = vec![];
//         for (ji, groups) in self.iter().enumerate() {
//             let mut curves = groups[0].clone();
//             for g1 in 1..groups.len() {
//                 curves = UnionBasis2::new(curves.clone(), groups[g1].clone());
//             }
//             results.push(curves);
//         }


// for (ji, groups) in self.iter().enumerate() {
        //     hit_miss[ji].push(vec![vec![]; groups.len()-1]);
        //     for g1 in 0..groups.len()-1 {
        //         for g0 in 0..=g1+1  {
        //             hit_miss[ji][g1].push(vec![HitMiss2::default(); groups[g0].len()]);
        //         }
        //     }
        // }




// let (starts, indexes) = job_indexes(&self);
        // let pairs = job_pairs(&starts, &self);

        // let mut pairs = vec![];
        // for (ji, job) in self.iter().enumerate() {
        //     for g1 in 1..job.len(){
        //         for g0 in 0..g1 {
        //             for c0 in 0..job[g0].len(){
        //                 for c1 in 0..job[g1].len(){
        //                     pairs.push((
        //                         jobs[ji] + groups[g0] + c0, 
        //                         jobs[ji] + groups[g1] + c1
        //                     ));
        //                 }  
        //             }   
        //         }
        //     }
        // }