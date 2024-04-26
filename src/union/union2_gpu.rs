use crate::{log, CurveHit, CurveShape, HitTester2, Shape, Spatial3, UnionBatch};
use crate::hit::{hit2_gpu::HitTest2, Miss, HitMiss2, Hit2};
use glam::*;

pub trait Union2 {
    fn union(self) -> Vec<Vec<CurveShape>>;
}

impl Union2 for Vec<Vec<Vec<CurveShape>>> { // job, stage, group
    fn union(self) -> Vec<Vec<CurveShape>> { 
        let batch = UnionBatch::new(&self);
        let curves: Vec<CurveShape> = self.clone().into_iter().flatten().flatten().collect();
        let (hits, misses) = curves.hit(&batch.pairs);
        let mut hit_miss: Vec<Vec<Vec<Vec<HitMiss2>>>> = vec![vec![]; self.len()];
        for (ji, groups) in self.iter().enumerate() {
            hit_miss[ji].push(vec![vec![]; groups.len()-1]);
            for g1 in 0..groups.len()-1 {
                for g0 in 0..=g1+1  {
                    hit_miss[ji][g1].push(vec![HitMiss2::default(); groups[g0].len()]);
                }
            }
        }
        for hit in &hits {
            let (ji, g0, i0, g1, i1) = batch.index(&hit.pair);
            hit_miss[ji][g1-1][g0][i0].hits.push(Hit2{u:hit.u0, dot:hit.dot0});
            hit_miss[ji][g1-1][g1][i1].hits.push(Hit2{u:hit.u1, dot:hit.dot1});
        }
        for miss in &misses {
            let (ji, g0, i0, g1, i1) = batch.index(&miss.pair);
            hit_miss[ji][g1-1][g0][i0].misses.push(Miss{dot:miss.dot0, distance:miss.distance});
            hit_miss[ji][g1-1][g1][i1].misses.push(Miss{dot:miss.dot1, distance:miss.distance});
        }
        let mut results = vec![];
        for (ji, groups) in self.iter().enumerate() {
            let mut curves = groups[0].clone();
            for g1 in 1..groups.len() {
                curves = UnionBasis2::new(curves.clone(), groups[g1].clone());
            }
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
    pub tester: HitTester2,
    pub groups: [Vec<CurveShape>; 2],
    pub hits:   [Vec<Vec<CurveHit>>; 2], 
    pub miss:   [Vec<Vec<Miss>>; 2], 
    pub curves: Vec<CurveShape>,
    pub shapes: Vec<Shape>,
}

impl UnionBasis2 { 
    pub fn new(curves0: Vec<CurveShape>, curves1: Vec<CurveShape>) -> Vec<CurveShape> {
        UnionBasis2 {
            tester: HitTester2 {
                curves: (CurveShape::default(), CurveShape::default()),
                spatial: Spatial3::new(), 
                points:  vec![],
            },
            hits: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
            miss: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
            groups: [curves0, curves1],
            curves: vec![],
            shapes: vec![],
        }.build()
    }

    pub fn build(&mut self) -> Vec<CurveShape> {
        self.test_groups();
        for g in 0..2 {
            for i in 0..self.groups[g].len() {
                if self.hits[g][i].is_empty() {
                    self.miss[g][i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                    if self.miss[g][i].is_empty() || self.miss[g][i][0].dot * self.groups[g][i].nurbs.sign < 0. { 
                        self.curves.push(self.groups[g][i].clone());   
                    }
                }else{
                    self.hits[g][i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
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
        for curve_hit in self.hits[g][i].iter() {
            if curve_hit.dot * curve.nurbs.sign > 0. {
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
        if self.hits[g][i].last().expect("There should be one or more hits.").dot * curve.nurbs.sign > 0. {
            self.curves.push(curve);
        }
    }

    fn test_groups(&mut self){
        for i0 in 0..self.groups[0].len() {
            for i1 in 0..self.groups[1].len() {
                self.tester.curves.0 = self.groups[0][i0].clone();
                self.tester.curves.1 = self.groups[1][i1].clone();
                for u0 in self.groups[0][i0].get_unique_knots() { 
                    for u1 in self.groups[1][i1].get_unique_knots() { 
                        self.test_curves(i0, i1, u0, u1);
                    }
                }
            }
        }
    }

    fn test_curves(&mut self, i0: usize, i1: usize, u0: f32, u1: f32) { 
        // if let Some(hit_miss) = self.tester.test(u0, u1) {
        //     match hit_miss {
        //         HitMiss2::Hit(hit) => {
        //             self.hits[0][i0].push(hit.hit.0);
        //         self.hits[1][i1].push(hit.hit.1);
        //         },
        //         HitMiss2::Miss(miss) => {
        //             self.miss[0][i0].push(miss.0);
        //             self.miss[1][i1].push(miss.1);
        //         }
        //     }
        // }
    }
}


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