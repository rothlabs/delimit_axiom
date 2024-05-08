use crate::actor::UnionCascade;
use crate::shape::*;
use crate::{log, Model, Models, Reshape, Groups};
use serde::{Deserialize, Serialize};
use glam::*;

//use self::{union2::union_job2, union3::UnionBasis3};


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Union {
    pub parts:         Vec<Model>,
    pub negated_parts: Vec<Model>,
    pub reshape:       Reshape,
}

impl Union {
    pub fn shapes(&self) -> Vec<Shape> {
        log("union");
        let mut shape_groups = self.parts.shape_groups();
        let neg_shape_groups = self.negated_parts.shape_groups().negated();
        shape_groups.extend(neg_shape_groups);
        shape_groups.union()
        //vec![shape_groups].union()[0].clone()
    }
}





        // let (_, facets, curve_groups_basis, facet_groups_basis) = get_grouped_curves_and_facets(&self.parts);
        // let (_, neg_facets, neg_curve_groups, neg_facet_groups) = get_grouped_curves_and_facets(&self.negated_parts);


// impl UnionIndexBatch {
//     pub fn new<T>(jobs: &Vec<Vec<Vec<T>>>) -> Self {
//         let (starts, indexes) = job_indexes(jobs);
//         UnionIndexBatch {
//             pairs: job_pairs(&starts, jobs),
//             indexes,
//         }
//     }
//     // pub fn hit_miss(&self) {
        
//     // }
//     pub fn index(&self, pair: &TestPair) -> (usize, usize, usize, usize, usize) {
//         let (_,  g0, i0) = self.indexes[pair.i0];
//         let (ji, g1, i1) = self.indexes[pair.i1];
//         (ji, g0, i0, g1, i1)
//     }
// }

// pub fn job_pairs<T>(starts: &[Vec<usize>; 2], jobs: &Vec<Vec<Vec<T>>>) -> Vec<TestPair> {
//     let mut pairs = vec![];
//     for (ji, groups) in jobs.iter().enumerate() {
//         for g1 in 1..groups.len(){
//             for g0 in 0..g1 {
//                 for c0 in 0..groups[g0].len(){
//                     for c1 in 0..groups[g1].len(){
//                         pairs.push(TestPair{
//                             i0: starts[0][ji] + starts[1][g0] + c0, 
//                             i1: starts[0][ji] + starts[1][g1] + c1,
//                             reverse: false,
//                         });
//                     }  
//                 }   
//             }
//         }
//     }
//     pairs
// }

















// if facets.is_empty() && neg_facets.is_empty() {
//     let mut groups = curve_groups_basis;
//     for neg_group in neg_curve_groups {
//         let mut group = vec![];
//         for mut curve in neg_group {
//             curve.negate();
//             group.push(curve);
//         }
//         groups.push(group);
//     }
//     let mut curves0 = groups.first().unwrap_or(&vec![]).clone();
//     // log("start compound gpu union2");
//     for curves1 in groups.iter().skip(1) {
//         // let mut basis = UnionBasis2::new(curves0, curves1.clone());
//         // curves0 = basis.build(); // shapes.extend(basis.shapes);
//         curves0 = [curves0, curves1.clone()].union();
//     }
//     //shapes.extend(curves0.iter().map(|c| Shape::Curve(c.clone())));
//     shapes.extend(curves0);
//     shapes
// }else{
//     let mut curve_groups = curve_groups_basis;
//     let mut facet_groups = facet_groups_basis;
//     for i in 0..neg_curve_groups.len() {
//         let mut curve_group = vec![];
//         let mut facet_group = vec![];
//         for mut curve in neg_curve_groups[i].clone() {
//             curve.negate();
//             curve_group.push(curve);
//         }
//         for mut facet in neg_facet_groups[i].clone() {
//             facet.negate();
//             facet_group.push(facet);
//         }
//         curve_groups.push(curve_group);
//         facet_groups.push(facet_group);
//     }
//     //UnionBasis3::get_shapes(curve_groups, facet_groups)
//     let mut shapes = facet_groups.union();
//     for curve_group in &curve_groups {
//         for curve in curve_group {
//             shapes.push(curve.clone());
//         }
//     }
//     shapes
// }






//let seed: [u8; 32] = *b"01234567891234560123456789123456";
//rng: StdRng::from_seed(seed),


// let mut curves0 = curve_groups.first().unwrap_or(&vec![]).clone();
            // let mut facets0 = facet_groups.first().unwrap_or(&vec![]).clone();
            // //self.get_hit_points(facet_groups.clone());
            // for i in 1..facet_groups.len() {
            //     let curves1 = curve_groups[i].clone();
            //     let facets1 = facet_groups[i].clone();
            //     let mut basis = UnionBasis3::new(curves0, curves1, facets0, facets1, tolerance, step);
            //     (curves0, facets0) = basis.build(i);
            //     shapes.extend(basis.shapes);
            // }
            // shapes.extend(curves0.iter().map(|c| Shape::Curve(c.clone())));
            // shapes.extend(facets0.iter().map(|f| Shape::Facet(f.clone())));


// let mut idx_texture: Vec<usize> = vec![];
        // let mut int_texture: Vec<usize> = vec![];
        // let mut f32_texture: Vec<f32> = vec![];
        // for (group_index, group) in groups.iter().enumerate() {
        //     for facet in group { // for (fi, facet) in group.iter().enumerate() {
        //         let int_index = int_texture.len();
        //         let f32_index = f32_texture.len();
        //         int_texture.extend(&[facet.controls.len(), facet.nurbs.order]);
        //         f32_texture.extend(&facet.nurbs.knots);
        //         f32_texture.extend(&facet.nurbs.weights);
        //         for curve in &facet.controls { //for (ci, curve) in facet.controls.iter().enumerate() {
        //             int_texture.extend(&[curve.controls.len(), curve.nurbs.order]);
        //             f32_texture.extend(&curve.nurbs.knots);
        //             f32_texture.extend(&curve.nurbs.weights);
        //             for point in &curve.controls { //for (pi, point) in curve.controls.iter().enumerate() {
        //                 idx_texture.extend(&[group_index, int_index, f32_index]);
        //                 f32_texture.extend(&point.to_array());
        //             }
        //         }
        //     }
        // }
        //get_facet_hit_points(idx_texture, int_texture, f32_texture);

// let mut basis = UnionBasis3 {
                //     tester: HitTester3 {
                //         curves: (CurveShape::default(), CurveShape::default()),
                //         facets: (FacetShape::default(), FacetShape::default()),
                //         spatial: Spatial3::new(step), // (0..facets.len()).map(|_| Spatial3::new(step)).collect(), // 
                //         points:  vec![],
                //         tolerance: 0.05,
                //         step,
                //     },
                //     facet_hits: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]], 
                //     facet_miss: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]],
                //     curve_groups: [curves0, curves1],
                //     facet_groups: [facets0, facets1],
                //     curves: vec![],
                //     facets: vec![],
                //     shapes: vec![],
                // };

// let mut basis = UnionBasis2 {
                //     tester: HitTester2 {
                //         curves: (CurveShape::default(), CurveShape::default()),
                //         spatial: Spatial3::new(duplication_tolerance), 
                //         points:  vec![],
                //         tolerance,
                //         duplication_tolerance,
                //     },
                //     hits: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
                //     miss: [vec![vec![]; curves0.len()], vec![vec![]; curves1.len()]],
                //     groups: [curves0, curves1.clone()],
                //     curves: vec![],
                //     shapes: vec![],
                // };

//let mut curve_params: HashMap<usize, CurveParams> = HashMap::new(); 
        // for (i, curve) in curves.iter().enumerate() {
        //     let (step, params) = curve.get_param_step_and_samples(1, cell_size);
        //     curve_params.insert(i, CurveParams{i, step, params});
        // }

        //let mut facet_params: HashMap<usize, FacetParams> = HashMap::new(); 
            // for (i, facet) in facets.iter().enumerate() {
            //     let (step, params) = facet.get_param_step_and_samples(1, cell_size);
            //     facet_params.insert(i, FacetParams{i, step, params});
            // }

// #[derive(Clone)]
// struct CurveParams {
//     i: usize,
//     step: f32,
//     params: Vec<f32>,
// }

// #[derive(Clone)]
// struct FacetParams {
//     i: usize,
//     step: Vec2,
//     params: Vec<Vec2>,
// }

// use rand::SeedableRng;
// use rand::rngs::StdRng; //use rand::rngs::SmallRng;



            //let params = curve.get_param_samples(4, cell_size);
            //let step = curve.get_param_step(4, cell_size);