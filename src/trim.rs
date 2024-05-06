use crate::{hit::{Hit2, HitJob, HitMiss2, Miss}, log, HitTest2, Shape, Shapes};

// pub trait Trim {
//     fn trim(self) -> Vec<Shape>;
// }

// impl Trim for Vec<Shape> {
//     fn trim(self) -> Vec<Shape> {
//         // CurveTrimmer {
//         //     tester: HitTester2 {
//         //         curves: (Shape::default(), Shape::default()),
//         //         spatial: Spatial3::new(DUP_0_TOL), 
//         //         points:  vec![],
//         //     },
//         //     hits: vec![vec![]; self.len()],
//         //     miss: vec![vec![]; self.len()],
//         //     group: self,
//         //     curves: vec![],
//         //     shapes: vec![],
//         // }.make()
//     }
// }

pub trait TrimJob { // TODO: rename to Union in different module from "Models" module
    fn trim(self) -> Vec<Vec<Shape>>;
}

impl TrimJob for Vec<Vec<Shape>> { // jobs, shapes
    fn trim(self) -> Vec<Vec<Shape>> { 
        let shapes0: Vec<Shape> = self.clone().into_iter().flatten().collect();
        if shapes0.high_rank() < 2 {
            trim_job2(self)
        }else{
            vec![]
        }
    }
}

fn trim_job2(jobs: Vec<Vec<Shape>>) -> Vec<Vec<Shape>> { 
    let batch = HitJob::new(&jobs);
    let shapes: Vec<Shape> = jobs.clone().into_iter().flatten().collect();
    let (hits2, misses) = shapes.hit2(&batch.pairs);
    let mut hits:   Vec<Vec<HitMiss2>> = vec![vec![]; jobs.len()];
    for (ji, shapes) in jobs.iter().enumerate() {
        hits[ji].extend(vec![HitMiss2::default(); shapes.len()]);
    }
    for hit in &hits2 {
        let (ji, i0, i1) = batch.index(&hit.pair);
        hits[ji][i0].hits.push(Hit2{u:hit.u0, dot:hit.dot0});
        hits[ji][i1].hits.push(Hit2{u:hit.u1, dot:hit.dot1});
    }
    for miss in &misses {
        let (ji, i0, i1) = batch.index(&miss.pair);
        hits[ji][i0].misses.push(Miss{dot:miss.dot0, distance:miss.distance});
        hits[ji][i1].misses.push(Miss{dot:miss.dot1, distance:miss.distance});
    }
    let mut results = vec![];
    for (ji, shapes0) in jobs.iter().enumerate() {
        let shapes1 = Trim2::shapes(&shapes0, &hits[ji]); 
        results.push(shapes1);
    }
    results
}

pub struct Trim2 {
    pub hits:   Vec<HitMiss2>, 
    pub group:  Vec<Shape>,
    pub curves: Vec<Shape>,
}

impl Trim2 { 
    pub fn shapes(group: &Vec<Shape>, hits: &Vec<HitMiss2>) -> Vec<Shape> { 
        Trim2 {
            hits: hits.clone(),
            group: group.clone(), 
            curves: vec![],
        }.make()
    }
    pub fn make(&mut self) -> Vec<Shape> {
        for i in 0..self.group.len() {
            if self.hits[i].hits.is_empty() {
                self.curves.push(self.group[i].clone());   
            }else{
                self.hits[i].hits.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
                self.add_bounded_curves(i);   
            }
        }
        self.curves.clone()
    }
    fn add_bounded_curves(&mut self, i: usize) {
        let mut curve = self.group[i].clone();
        let min_basis = curve.basis.min;
        for hit in self.hits[i].hits.iter() { 
            if hit.dot * curve.basis.sign > 0. {
                curve.basis.set_min(hit.u);
            }else{
                curve.basis.set_max(min_basis, hit.u);
                let range = curve.basis.range();
                if range < 0.001 {
                    console_log!("trim range: {}", range);
                }
                self.curves.push(curve);
                curve = self.group[i].clone();
            }
        }
        if self.hits[i].hits.last().expect("There should be one or more hits.").dot * curve.basis.sign > 0. {
            let range = curve.basis.range();
            if range < 0.001 {
                console_log!("trim range last: {}", range);
            }
            self.curves.push(curve);
        }
    }
}


// fn test_groups(&mut self){
//     for i0 in 0..self.group.len() {
//         for i1 in i0..self.group.len() {
//             if i0 == i1 {continue}
//             self.tester.curves.0 = self.group[i0].clone();
//             self.tester.curves.1 = self.group[i1].clone();
//             for u0 in self.group[i0].get_unique_knots() { 
//                 for u1 in self.group[i1].get_unique_knots() { 
//                     self.test_curves(i0, i1, u0, u1);
//                 }
//             }
//         }
//     }
// }

// fn test_curves(&mut self, i0: usize, i1: usize, u0: f32, u1: f32) { 
//     if let Some(hit_miss) = self.tester.test(u0, u1) {
//         match hit_miss {
//             HitMiss2::Hit(hit) => {
//                 self.hits[i0].push(hit.hit.0);
//                 self.hits[i1].push(hit.hit.1);
//             },
//             HitMiss2::Miss(miss) => {
//                 self.miss[i0].push(miss.0);
//                 self.miss[i1].push(miss.1);
//             }
//         }
//     }
// }