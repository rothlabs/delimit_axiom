use crate::shape::*;
use crate::hit::groups::HitTestGroups;

pub fn union_job3(jobs: Vec<Vec<Vec<Shape>>>) -> Vec<Vec<Shape>> {
    //let score = jobs.hit();
    for j in 0..jobs.len() {
        
    }

    let mut results = vec![];
    // for (ji, groups) in jobs.iter().enumerate() {
    //     let mut shapes0 = groups[0];
    //     for shapes1 in groups.iter().skip(1) {
    //         shapes0 = UnionBasis3::get_shapes(&[shapes0, shapes1], &hits[ji]); 
    //     }
    //     results.push(shapes);
    // }
    results
    // UnionBasis3::get_shapes(vec![self])
}

// pub struct UnionBasis3 {
//     hits3: Vec<HitPair3>,
//     misses3: Vec<MissPair>,
//     pub facet_groups: Vec<Vec<Shape>>,
//     hit_groups: Vec<Vec<Vec<Shape>>>,
//     //indexes: Vec<(usize, usize, usize)>,
//     pub batch: CascadeGroupJob,
//     pub shapes: Vec<Shape>,
// }

// impl UnionBasis3 { 
//     pub fn get_shapes(jobs: Vec<Vec<Vec<Shape>>>) -> Vec<Shape> {
//         //let mut wow = vec![];
//         let batch = CascadeGroupJob::new(&jobs);
//         let facet_groups = jobs[0].clone();
//         let facets: Vec<Shape> = facet_groups.clone().into_iter().flatten().collect();
//         let (hits3, misses3) = facets.hit(&batch.pairs); 
//         UnionBasis3 {
//             hits3,
//             misses3,
//             facet_groups,
//             hit_groups: vec![],
//             batch,
//             shapes: vec![],
//         }.make_shapes()
//     }

//     pub fn make_shapes(&mut self) -> Vec<Shape> {
//         //let hits = self.hit_basis.facet_hits.clone();
//         //let mut misses = self.misses3.clone();
//                 //self.shapes = self.basis.shapes.clone();
//         //let mut collect_facet: Vec<bool> = vec![true; self.facets.len()];
//         //let mut hits_len: Vec<usize> = vec![0; self.facets.len()];
//         //let mut hit_groups = vec![];// facet_groups_len vec![vec![]; facet_groups_len];
//         let mut miss_groups: Vec<Vec<Vec<Miss>>> = vec![]; // vec![vec![]; self.facet_groups.len()-1];
//         let mut collect_groups = vec![];
//         //for hi in 0..self.facet_groups.len()-1 {
//         for (gi, group) in self.facet_groups.iter().enumerate() {
//             self.hit_groups.push(vec![vec![]; group.len()]);
//             miss_groups.push(vec![vec![]; group.len()]);
//             collect_groups.push(vec![true; group.len()]);
//         }
//         //}
//         for hit in &self.hits3 {
//             self.shapes.push(hit.curve2.clone());
//             let (ji, g0, f0, g1, f1) = self.batch.index(&hit.pair);
//             self.hit_groups[g0][f0].push(hit.curve0.clone());
//             self.hit_groups[g1][f1].push(hit.curve1.clone());
//         }
//         for MissPair{pair, dot0, dot1, distance} in &self.misses3 {
//             let (ji, g0, f0, g1, f1) = self.batch.index(&pair);
//             miss_groups[g0][f0].push(Miss{distance:*distance, dot:*dot0});
//             miss_groups[g1][f1].push(Miss{distance:*distance, dot:*dot1});
//         }
        
//         for gi in 0..self.facet_groups.len() {
//             for fi in 0..self.facet_groups[gi].len() {
//                 if collect_groups[gi][fi] {
//                     if self.hit_groups[gi][fi].is_empty() {
//                         //self.move_misses_in_bounds(gi, fi, hi); 
//                         miss_groups[gi][fi].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
//                         if !miss_groups[gi][fi].is_empty() && miss_groups[gi][fi][0].dot * self.facet_groups[gi][fi].basis.sign > 0. {   
//                             collect_groups[gi][fi] = false;
//                         }
//                     }else{
//                         self.union_facet_with_hits(gi, fi);  
//                     }
//                     // if self.facet_groups[gi][fi].nurbs.sign < 0. {
//                     //     //signs[gi][fi] = -1.;
//                     //     let facet = self.facet_groups[gi].get_mut(fi).unwrap();
//                     //     facet.reverse().negate();
//                     //     // for gi1 in (gi+1)..self.facet_groups.len() {
//                     //     //     for bndry in &mut self.hit_groups[gi1][fi] {
//                     //     //         bndry.reshape(Mat4::from_translation(vec3(0., 1., 0.)) * Mat4::from_scale(vec3(1., -1., 1.)));
//                     //     //     }
//                     //     // }
//                     // }
//                 }
//             }
//         }
//         for gi in 0..self.facet_groups.len() {
//             for fi in 0..self.facet_groups[gi].len() {
//                 if collect_groups[gi][fi] {
//                     let mut facet = self.facet_groups[gi][fi].clone();
//                     if facet.basis.sign < 0. {facet.reverse().negate();}
//                     self.shapes.push(facet);
//                 }
//             }
//         }
//         self.shapes.clone()
//     }

//     fn move_misses_in_bounds(&mut self, gi: usize, fi: usize, hi: usize) {

//     }

//     fn union_facet_with_hits(&mut self, gi: usize, fi: usize) {
//         let facet = self.facet_groups[gi].get_mut(fi).expect("Should be a facet at this index.");
//         if facet.basis.sign < 0. {
//             for curve in &mut facet.boundaries {
//                 curve.negate();
//             }
//         }
//         // for j in 0..facet.boundaries.len() {
//         //     let mut bndry = facet.boundaries[j].clone();
//         //     bndry.controls.clear();
//         //     for k in 0..facet.boundaries[j].controls.len() {
//         //         bndry.controls.push(rank0(facet.boundaries[j].controls[k].point(&[]) + vec3(
//         //             100. + fi as f32 * 2., // + (j as f32)*0.005,  
//         //             gi as f32 * 2., // + (j as f32)*0.01, 
//         //             0.
//         //         )));
//         //     }
//         //     self.shapes.push(bndry);
//         // }
//         // for j in 0..self.hit_groups[gi][fi].len() {
//         //     let mut bndry = self.hit_groups[gi][fi][j].clone();
//         //     bndry.controls.clear();
//         //     for k in 0..self.hit_groups[gi][fi][j].controls.len() {
//         //         bndry.controls.push(rank0(self.hit_groups[gi][fi][j].controls[k].point(&[]) + vec3(
//         //             100. + fi as f32 * 2.,// + (j as f32)*0.01,  
//         //             gi as f32 * 2.,//  + (j as f32)*0.01, 
//         //             0.
//         //         )));
//         //     }
//         //     self.shapes.push(bndry);
//         // }
//         let trimmed =  vec![self.hit_groups[gi][fi].clone()].trim()[0].clone(); // Trim::new(self.hit_basis.facet_hits[gi][fi][hi].clone()); // 0.001
//         // for j in 0..trimmed.len() {
//         //     let mut bndry = trimmed[j].clone();
//         //     bndry.controls.clear();
//         //     for k in 0..trimmed[j].controls.len() {
//         //         bndry.controls.push(rank0(trimmed[j].controls[k].point(&[]) + vec3(
//         //             100. + fi as f32 * 2., //  + (j as f32)*0.01  
//         //             gi as f32 * 2., //  + (j as f32)*0.01 
//         //             0.
//         //         )));
//         //     }
//         //     self.shapes.push(bndry);
//         // }
//         // let mut union = UnionBasis2::new(facet.boundaries.clone(), trimmed.clone()); // self.facet_hits[g][i].clone()
//         // facet.boundaries = union.build();
//         facet.boundaries = facet.boundaries.union(&trimmed); //vec![facet.boundaries.clone(), trimmed.clone()].union();
//         //if gi < 1 {
//             for j in 0..facet.boundaries.len() {
//                 let mut bndry = facet.boundaries[j].clone();
//                 bndry.controls.clear();
//                 for k in 0..facet.boundaries[j].controls.len() {
//                     bndry.controls.push(rank0(facet.boundaries[j].controls[k].point(&[]) + vec3(
//                         100. + fi as f32 * 2.,// + (j as f32)*0.005,  
//                         gi as f32 * 2.,// + (j as f32)*0.01, 
//                         0.
//                     )));
//                 }
//                 self.shapes.push(bndry);
//             }
//         //}
//     }
// }





// pub fn make_shapes(&mut self) -> Vec<Shape> {
//     self.hit_basis.make().expect("Facet intersection should succeed for union3 to work.");
//     let hits = self.hit_basis.facet_hits.clone();
//     let mut misses = self.hit_misses3.clone();
//     self.shapes = self.hit_basis.shapes.clone();
//     for gi in 0..self.facet_groups.len() {
//         for fi in 0..self.facet_groups[gi].len() {
//             let mut collect_facet = false;
//             for hi in 0..hits[gi][fi].len() {
//                 if hits[gi][fi][hi].is_empty() {
//                     self.move_misses_in_bounds(gi, fi, hi); 
//                     misses[gi][fi][hi].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
//                     if misses[gi][fi][hi].is_empty() || misses[gi][fi][hi][0].dot * self.facet_groups[gi][fi].nurbs.sign < 0. {   
//                         collect_facet = true;
//                     }else{
//                         collect_facet = false;
//                         break; // This should ensure the facet is not collected in later hit groups
//                     }
//                 }else{
//                     self.union_facet_with_hits(gi, fi, hi);  
//                     collect_facet = true;
//                 }
//             }
//             if collect_facet {
//                 let mut facet = self.facet_groups[gi][fi].clone();
//                 if facet.nurbs.sign < 0. {facet.reverse().negate();}
//                 self.shapes.push(Shape::Facet(facet));
//             }
//         }
//     }
//     for curve_group in &self.curve_groups {
//         for curve in curve_group {
//             self.shapes.push(Shape::Curve(curve.clone()));
//         }
//     }
//     self.shapes.clone()
// }



// fn union_facet_with_hits(&mut self, gi: usize, fi: usize, hi: usize) {
//     let facet = self.facet_groups[gi].get_mut(fi).expect("Should be a facet at this index.");
//     if facet.nurbs.sign < 0. {
//         for curve in &mut facet.boundaries {
//             curve.negate();
//         }
//     }
//     // for j in 0..facet.boundaries.len() {
//     //     let mut bndry = facet.boundaries[j].clone();
//     //     bndry.controls.clear();
//     //     for k in 0..facet.boundaries[j].controls.len() {
//     //         bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
//     //             100. + fi as f32 * 2.,// + (j as f32)*0.005,  
//     //             gi as f32 * 2.,// + (j as f32)*0.01, 
//     //             0.
//     //         ));
//     //     }
//     //     self.shapes.push(Shape::Curve(bndry));
//     // }
//     // for j in 0..self.hit_basis.facet_hits[gi][fi][hi].len() {
//     //     let mut bndry = self.hit_basis.facet_hits[gi][fi][hi][j].clone();
//     //     bndry.controls.clear();
//     //     for k in 0..self.hit_basis.facet_hits[gi][fi][hi][j].controls.len() {
//     //         bndry.controls.push(self.hit_basis.facet_hits[gi][fi][hi][j].controls[k] + vec3(
//     //             100. + fi as f32 * 2.,// + (j as f32)*0.01,  
//     //             gi as f32 * 2.,//  + (j as f32)*0.01, 
//     //             0.
//     //         ));
//     //     }
//     //     self.shapes.push(Shape::Curve(bndry));
//     // }
//     let trimmed =  self.hit_basis.facet_hits[gi][fi][hi].clone().trim();// Trim::new(self.hit_basis.facet_hits[gi][fi][hi].clone()); // 0.001
//     // for j in 0..trimmed.len() {
//     //     let mut bndry = trimmed[j].clone();
//     //     bndry.controls.clear();
//     //     for k in 0..trimmed[j].controls.len() {
//     //         bndry.controls.push(trimmed[j].controls[k] + vec3(
//     //             100. + fi as f32 * 2., //  + (j as f32)*0.01  
//     //             gi as f32 * 2., //  + (j as f32)*0.01 
//     //             0.
//     //         ));
//     //     }
//     //     self.shapes.push(Shape::Curve(bndry));
//     // }
//     let mut union = UnionBasis2::new(facet.boundaries.clone(), trimmed.clone()); // self.facet_hits[g][i].clone()
//     facet.boundaries = union.build();
//     for j in 0..facet.boundaries.len() {
//         let mut bndry = facet.boundaries[j].clone();
//         bndry.controls.clear();
//         for k in 0..facet.boundaries[j].controls.len() {
//             bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
//                 100. + fi as f32 * 2.,// + (j as f32)*0.005,  
//                 gi as f32 * 2.,// + (j as f32)*0.01, 
//                 0.
//             ));
//         }
//         self.shapes.push(Shape::Curve(bndry));
//     }
// }