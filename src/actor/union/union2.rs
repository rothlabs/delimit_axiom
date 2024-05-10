use crate::Shape;
use crate::hit::{ClosetOut, Score};
use crate::hit::groups::HitTest;
use crate::actor::Trim;

pub fn union_job2(jobs: Vec<[Vec<Shape>; 2]>) -> Vec<Vec<Shape>> { 
    let hits = jobs.hit();
    let mut shapes = vec![];
    for i in 0..jobs.len() {
        shapes.push(union2(
            jobs[i].iter().flatten().collect(),
            hits[i].iter().flatten().collect()
        ));
    }
    shapes
}

fn union2(shapes0: Vec<&Shape>, score: Vec<&Score>) -> Vec<Shape> {
    let mut shapes1 = vec![];
    for i in 0..shapes0.len() {
        if score[i].hits.is_empty() {
            if score[i].outs.is_empty() {
                shapes1.push(shapes0[i].clone());  
                continue;
            }
            let out = score[i].outs.closest();
            if shapes0[i].basis.sign * out.dot > 0. { 
                shapes1.push(shapes0[i].clone());   
            }
        }else{
            let mut hits = score[i].hits.clone();
            hits.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap()); 
            shapes1.extend(shapes0[i].trim(&hits));
        }
    }
    for shape in &mut shapes1 {
        if shape.basis.sign < 0. {
            shape.reverse().negate();
        }
    }
    shapes1
}


// let mut dot = 0.;
//             let mut distance = INFINITY;
//             for miss in &hit_miss[i].misses {
//                 if miss.distance < distance {
//                     dot = miss.dot;
//                     distance = miss.distance;
//                 }
//             }



// let curves = Union2 {
        //     hits:     hits[ji].clone().into_iter().flatten().collect(),//[hits[ji][0].clone(), hits[ji][1].clone()],
        //     shapes0:  groups.clone().into_iter().flatten().collect(),//, 
        //     shapes: vec![],
        // }.shapes();


            // let misses = hit_miss[i].misses.sort_by(|a, b| 
            //     a.distance.partial_cmp(&b.distance).unwrap()
            // );
// let miss = hit_miss[i].misses.iter().fold(0., |distance, miss| 
//     miss.distance.min(distance)
// );




// pub struct Union2 {
//     pub shapes0: Vec<Shape>, // &'static 
//     pub hits:    Vec<HitMiss>, 
//     pub shapes:  Vec<Shape>,
// }

// impl Union2 { 
//     pub fn shapes(&mut self) -> Vec<Shape> {
//         for i in 0..self.shapes0.len() {
//             if self.hits[i].hits.is_empty() {
//                 self.hits[i].misses.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
//                 if self.hits[i].misses.is_empty() || self.hits[i].misses[0].dot * self.shapes0[i].basis.sign > 0. { 
//                     self.shapes.push(self.shapes0[i].clone());   
//                 }
//             }else{
//                 self.hits[i].hits.sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap()); 
//                 self.shapes.extend(
//                     self.shapes0[i].trim(&self.hits[i].hits)
//                 );
//             }
//         }
//         for shape in &mut self.shapes {
//             if shape.basis.sign < 0. {
//                 shape.reverse().negate();
//             }
//         }
//         self.shapes.clone()
//     }
// }







    // fn add_bounded_curves(&mut self, g: usize, i: usize) {
    //     let mut curve = self.groups[g][i].clone();
    //     let min_basis = curve.basis.min;
    //     for curve_hit in self.hits[g][i].hits.iter() {
    //         if curve_hit.dot * curve.basis.sign < 0. {
    //             curve.basis.set_min(curve_hit.u);
    //         }else{
    //             let range = curve_hit.u - min_basis;
    //             if range < 0.0001 {
    //                 console_log!("union2 range: {}", range);
    //             }
    //             curve.basis.set_max(min_basis, curve_hit.u);
    //             self.shapes.push(curve);
    //             curve = self.groups[g][i].clone();
    //         }
    //     }
    //     if self.hits[g][i].hits.last().expect("There should be one or more hits.").dot * curve.basis.sign < 0. {
    //         self.shapes.push(curve);
    //     }
    // }






    // let batch = CascadeGroupJob::new(&jobs);
    // let shapes: Vec<Shape> = jobs.clone().into_iter().flatten().flatten().collect();
    // let (hit_pairs, miss_pairs) = shapes.hit2(&batch.pairs);
    // let mut hits: Vec<Vec<Vec<HitMiss>>> = vec![vec![]; jobs.len()];
    // for (ji, groups) in jobs.iter().enumerate() {
    //     for gi in 0..groups.len() {
    //         hits[ji].push(vec![HitMiss::default(); groups[gi].len()]);
    //     }
    // }
    // for hit in &hit_pairs {
    //     let (ji, g0, i0, g1, i1) = batch.index(&hit.pair);
    //     hits[ji][g0][i0].hits.push(hit.hits.0.clone());
    //     hits[ji][g1][i1].hits.push(hit.hits.1.clone());
    // }
    // for miss in &miss_pairs {
    //     let (ji, g0, i0, g1, i1) = batch.index(&miss.pair);
    //     hits[ji][g0][i0].misses.push(Miss{dot:miss.dots.0, distance:miss.distance});
    //     hits[ji][g1][i1].misses.push(Miss{dot:miss.dots.1, distance:miss.distance});
    // }





// pub trait UnionCurves2 {
//     fn union(self) -> Vec<CurveShape>;
// }

// impl UnionCurves2 for [Vec<CurveShape>; 2] { 
//     fn union(self) -> Vec<CurveShape> {
//         let mut shapes0 = vec![];
//         for curve in self[0].clone() {
//                 shapes0.push(Shape::Curve(curve));
//         }
//         let mut shapes1 = vec![];
//         for curve in self[1].clone() {
//             shapes1.push(Shape::Curve(curve));
//         }
//         let mut curves = vec![];
//         for shape in vec![vec![shapes0, shapes1]].union()[0].clone() {
//             if let Shape::Curve(curve) = shape {
//                 curves.push(curve);
//             }
//         }
//         curves
//     }
// }




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