mod union2;
mod union3;

use crate::{log, get_grouped_curves_and_facets, CurveShape, FacetShape, Reshape, HitTester3, Model, Shape, Spatial3};
use serde::{Deserialize, Serialize};
use glam::*;

use self::{union2::UnionBasis2, union3::UnionBasis3};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Union::default")]
pub struct Union {
    pub parts:         Vec<Model>,
    pub negated_parts: Vec<Model>,
    pub reshape:     Reshape,
}

// pub struct UnionBasis {
//     pub parts:         Vec<Model>,
//     pub negated_parts: Vec<Model>,
//     pub transform:     Group,
// }

impl Union {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let tolerance = 0.005;
        let duplication_tolerance = tolerance * 10.;
        let (_, facets, curve_groups_basis, facet_groups_basis) = get_grouped_curves_and_facets(&self.parts);
        let (_, neg_facets, neg_curve_groups, neg_facet_groups) = get_grouped_curves_and_facets(&self.negated_parts);
        if facets.is_empty() && neg_facets.is_empty() {
            let mut groups = curve_groups_basis;
            for neg_group in neg_curve_groups {
                let mut group = vec![];
                for mut curve in neg_group {
                    curve.negate();
                    group.push(curve);
                }
                groups.push(group);
            }
            let mut curves0 = groups.first().unwrap_or(&vec![]).clone();
            for curves1 in groups.iter().skip(1) {
                let mut basis = UnionBasis2::new(curves0, curves1.clone(), tolerance, false);
                curves0 = basis.build();
                shapes.extend(basis.shapes);
            }
            shapes.extend(curves0.iter().map(|c| Shape::Curve(c.clone())));
        }else{
            let step = 2.;
            let mut curve_groups = curve_groups_basis;
            let mut facet_groups = facet_groups_basis;
            for i in 0..neg_curve_groups.len() {
                let mut curve_group = vec![];
                let mut facet_group = vec![];
                for mut curve in neg_curve_groups[i].clone() {
                    curve.negate();
                    curve_group.push(curve);
                }
                for mut facet in neg_facet_groups[i].clone() {
                    facet.negate();
                    facet_group.push(facet);
                }
                curve_groups.push(curve_group);
                facet_groups.push(facet_group);
            }
            let mut curves0 = curve_groups.first().unwrap_or(&vec![]).clone();
            let mut facets0 = facet_groups.first().unwrap_or(&vec![]).clone();
            for i in 1..curve_groups.len() {
                let curves1 = curve_groups[i].clone();
                let facets1 = facet_groups[i].clone();
                let mut basis = UnionBasis3 {
                    tester: HitTester3 {
                        curves: (CurveShape::default(), CurveShape::default()),
                        facets: (FacetShape::default(), FacetShape::default()),
                        spatial: Spatial3::new(step), // (0..facets.len()).map(|_| Spatial3::new(step)).collect(), // 
                        points:  vec![],
                        tolerance: 0.05,
                        step,
                    },
                    facet_hits: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]], 
                    facet_miss: [vec![vec![]; facets0.len()], vec![vec![]; facets1.len()]],
                    curve_groups: [curves0, curves1],
                    facet_groups: [facets0, facets1],
                    curves: vec![],
                    facets: vec![],
                    shapes: vec![],
                };
                (curves0, facets0) = basis.build();
                shapes.extend(basis.shapes);
            }
            shapes.extend(curves0.iter().map(|c| Shape::Curve(c.clone())));
            shapes.extend(facets0.iter().map(|f| Shape::Facet(f.clone())));
        }
        shapes
    }
}


//let seed: [u8; 32] = *b"01234567891234560123456789123456";
//rng: StdRng::from_seed(seed),



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