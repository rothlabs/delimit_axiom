mod union2;
mod union3;

use std::collections::HashMap;
use crate::{get_curves, get_curves_and_facets, get_grouped_curves_and_facets, group, log, FacetShape, Group, Hit3, HitTester3, Model, Shape, Spatial3};
use serde::{Deserialize, Serialize};
use glam::*;

use self::{union2::UnionBasis2, union3::UnionBasis3};

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Union::default")]
pub struct Union {
    pub parts: Vec<Model>,
    pub transform: Group,
}

impl Union {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let cell_size = 4.;
        let hit_step = 2.;
        let (curves, facets, grouped_curves, grouped_facets) = get_grouped_curves_and_facets(&self.parts);
        //let mut curve_params: HashMap<usize, CurveParams> = HashMap::new(); 
        // for (i, curve) in curves.iter().enumerate() {
        //     let (step, params) = curve.get_param_step_and_samples(1, cell_size);
        //     curve_params.insert(i, CurveParams{i, step, params});
        // }
        if facets.is_empty(){
            let mut basis = UnionBasis2 {
                curve_index0: 0,
                curve_index1: 0,
                hits: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                grouped_curves,
                //curve_params,
                cell_size,
                tolerance: 0.05,
                max_walk_iterations: 1000,
                //samples: vec![],
                shapes: vec![],
            };
            basis.get_shapes()
        }else{
            //let mut facet_params: HashMap<usize, FacetParams> = HashMap::new(); 
            // for (i, facet) in facets.iter().enumerate() {
            //     let (step, params) = facet.get_param_step_and_samples(1, cell_size);
            //     facet_params.insert(i, FacetParams{i, step, params});
            // }
            //let seed: [u8; 32] = *b"01234567891234560123456789123456";
            let mut basis = UnionBasis3 {
                hit3: HitTester3 {
                    curves: curves.clone(),
                    facets: facets.clone(),
                    facet_index0: 0,
                    facet_index1: 0,
                    hit_step,
                    hit_map:    (0..facets.len()).map(|_| Spatial3::new(hit_step)).collect(),
                    hit_points: (0..facets.len()).map(|_| vec![]).collect(),
                    tolerance: 0.05,
                },
                //rng: StdRng::from_seed(seed),
                facet_hits: (0..curves.len()).map(|_| vec![]).collect(),
                //curve_hits: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                facets,
                grouped_curves,
                grouped_facets,
                //tolerance: 0.05,
                max_walk_iterations: 800,
                shapes: vec![],
            };
            self.transform.get_reshapes(basis.get_shapes())
        }
    }
}

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