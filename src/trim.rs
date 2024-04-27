use crate::{hit::Miss, log, CurveHit, CurveShape, HitMiss2, HitTester2, Shape, Spatial3, DUP_TOL};
use glam::*;

pub trait Trim {
    fn trim(self) -> Vec<CurveShape>;
}

impl Trim for Vec<CurveShape> {
    fn trim(self) -> Vec<CurveShape> {
        CurveTrimmer {
            tester: HitTester2 {
                curves: (CurveShape::default(), CurveShape::default()),
                spatial: Spatial3::new(DUP_TOL), 
                points:  vec![],
            },
            hits: vec![vec![]; self.len()],
            miss: vec![vec![]; self.len()],
            group: self,
            curves: vec![],
            shapes: vec![],
        }.make()
    }
}

pub struct CurveTrimmer {
    pub tester: HitTester2,
    pub group:  Vec<CurveShape>,
    pub hits:   Vec<Vec<CurveHit>>, 
    pub miss:   Vec<Vec<Miss>>, 
    pub curves: Vec<CurveShape>,
    pub shapes: Vec<Shape>,
}

impl CurveTrimmer { 
    pub fn make(&mut self) -> Vec<CurveShape> {
        self.test_groups();
        for i in 0..self.group.len() {
            if self.hits[i].is_empty() {
                self.curves.push(self.group[i].clone());   
            }else{
                self.hits[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
                self.add_bounded_curves(i);   
            }
        }
        self.curves.clone()
    }

    fn add_bounded_curves(&mut self, i: usize) {
        let mut curve = self.group[i].clone();
        let min_basis = curve.min;
        for curve_hit in self.hits[i].iter() { 
            if curve_hit.dot * curve.nurbs.sign > 0. {
                curve.set_min(curve_hit.u);
            }else{
                let range = curve_hit.u - min_basis;
                if range < 0.0001 {
                    console_log!("trim range: {}", range);
                }
                curve.set_max(min_basis, curve_hit.u);
                self.curves.push(curve);
                curve = self.group[i].clone();
            }
        }
        if self.hits[i].last().expect("There should be one or more hits.").dot * curve.nurbs.sign > 0. {
            self.curves.push(curve);
        }
    }

    fn test_groups(&mut self){
        for i0 in 0..self.group.len() {
            for i1 in i0..self.group.len() {
                if i0 == i1 {continue}
                self.tester.curves.0 = self.group[i0].clone();
                self.tester.curves.1 = self.group[i1].clone();
                for u0 in self.group[i0].get_unique_knots() { 
                    for u1 in self.group[i1].get_unique_knots() { 
                        self.test_curves(i0, i1, u0, u1);
                    }
                }
            }
        }
    }

    fn test_curves(&mut self, i0: usize, i1: usize, u0: f32, u1: f32) { 
        if let Some(hit_miss) = self.tester.test(u0, u1) {
            match hit_miss {
                HitMiss2::Hit(hit) => {
                    self.hits[i0].push(hit.hit.0);
                    self.hits[i1].push(hit.hit.1);
                },
                HitMiss2::Miss(miss) => {
                    self.miss[i0].push(miss.0);
                    self.miss[i1].push(miss.1);
                }
            }
        }
    }
}
