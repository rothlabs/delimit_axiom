use std::convert::TryInto;

use crate::{log, CurveHit, CurveShape, Curves, HitTester2, Shape, Spatial3, UnionBatch};
use crate::hit::{hit2_gpu::HitTest2, Miss, HitMiss2, Hit2};
use glam::*;

pub trait UnionCurves2 {
    fn union(self) -> Vec<CurveShape>;
}

impl UnionCurves2 for [Vec<CurveShape>; 2] { 
    fn union(self) -> Vec<CurveShape> {
        let mut shapes0 = vec![];
        for curve in self[0].clone() {
                shapes0.push(Shape::Curve(curve));
        }
        let mut shapes1 = vec![];
        for curve in self[1].clone() {
            shapes1.push(Shape::Curve(curve));
        }
        let mut curves = vec![];
        for shape in vec![vec![shapes0, shapes1]].union()[0].clone() {
            if let Shape::Curve(curve) = shape {
                curves.push(curve);
            }
        }
        curves
    }
}

pub trait UnionBatch2 {
    fn union(self) -> Vec<Vec<Shape>>;
}

impl UnionBatch2 for Vec<Vec<Vec<Shape>>> { // jobs, groups, curves
    fn union(self) -> Vec<Vec<Shape>> { 
        let batch = UnionBatch::new(&self);
        let shapes: Vec<Shape> = self.clone().into_iter().flatten().flatten().collect();
        let (hits2, misses) = shapes.hit2(&batch.pairs);
        let mut hits:   Vec<[Vec<HitMiss2>; 2]> = vec![[vec![], vec![]]; self.len()];
        for (ji, groups) in self.iter().enumerate() {
            hits[ji][0].extend(vec![HitMiss2::default(); groups[0].len()]);
            hits[ji][1].extend(vec![HitMiss2::default(); groups[1].len()]);
        }
        for hit in &hits2 {
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
            let curves = UnionBasis2::shape([&groups[0], &groups[1]], &hits[ji]); 
            results.push(curves);
        }
        results
    }
}

pub struct UnionBasis2 {
    pub groups: [Vec<CurveShape>; 2],
    pub hits:   [Vec<HitMiss2>; 2], 
    pub curves: Vec<CurveShape>,
    pub shapes: Vec<Shape>,
}

impl UnionBasis2 { 
    pub fn shape(groups: [&Vec<Shape>; 2], hits: &[Vec<HitMiss2>; 2]) -> Vec<Shape> { 
        UnionBasis2 {
            hits: hits.clone(),
            groups: [groups[0].curves(), groups[1].curves()], 
            curves: vec![],
            shapes: vec![],
        }.build()
    }

    pub fn build(&mut self) -> Vec<Shape> {
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
            if curve.nurbs.sign < 0. {
                curve.reverse().negate();
            }
            self.shapes.push(Shape::Curve(curve.clone()));
        }
        self.shapes.clone()
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