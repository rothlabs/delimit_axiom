mod union2;
mod hit2;
mod union3;
mod hit3;

use std::collections::HashMap;
use crate::{get_curves, get_curves_and_facets, get_grouped_curves_and_facets, group, log, FacetShape, Model, Shape, Spatial2, Spatial3};
use rand::SeedableRng;
use rand::rngs::StdRng; //use rand::rngs::SmallRng;
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
}

impl Union {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let cell_size = 4.;
        let hit_step = 2.;
        let (curves, facets, grouped_curves, grouped_facets) = get_grouped_curves_and_facets(&self.parts);
        let mut curve_params: HashMap<usize, CurveParams> = HashMap::new(); 
        // for (i, curve) in curves.iter().enumerate() {
        //     let (step, params) = curve.get_param_step_and_samples(1, cell_size);
        //     curve_params.insert(i, CurveParams{i, step, params});
        // }
        // if facets.is_empty(){
        //     let mut basis = UnionBasis2 {
        //         hits: (0..curves.len()).map(|_| vec![]).collect(),
        //         curves,
        //         curve_params,
        //         cell_size,
        //         tolerance: 0.05,
        //         max_walk_iterations: 1000,
        //         samples: vec![],
        //         shapes: vec![],
        //     };
        //     basis.get_shapes()
        // }else{
            let mut facet_params: HashMap<usize, FacetParams> = HashMap::new(); 
            // for (i, facet) in facets.iter().enumerate() {
            //     let (step, params) = facet.get_param_step_and_samples(1, cell_size);
            //     facet_params.insert(i, FacetParams{i, step, params});
            // }
            //let seed: [u8; 32] = *b"01234567891234560123456789123456";
            let mut basis = UnionBasis3 {
                // facet0: FacetShape::default(),
                // facet1: FacetShape::default(),
                facet_index0: 0,
                facet_index1: 0,
                //rng: StdRng::from_seed(seed),
                hit_map: (0..facets.len()).map(|_| Spatial3::new(hit_step)).collect(),
                hit_points: (0..facets.len()).map(|_| vec![]).collect(),
                hit_step,
                facet_hits: (0..curves.len()).map(|_| vec![]).collect(),
                curve_hits: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                facets,
                grouped_curves,
                grouped_facets,
                curve_params,
                facet_params,
                cell_size,
                tolerance: 0.05,
                max_walk_iterations: 800,
                curve_samples: vec![],
                facet_samples: vec![],
                shapes: vec![],
            };
            basis.get_shapes()
        //}
    }
}

#[derive(Clone)]
struct CurveParams {
    i: usize,
    step: f32,
    params: Vec<f32>,
}

#[derive(Clone)]
struct FacetParams {
    i: usize,
    step: Vec2,
    params: Vec<Vec2>,
}



            //let params = curve.get_param_samples(4, cell_size);
            //let step = curve.get_param_step(4, cell_size);