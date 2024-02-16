mod union2;
mod intersection2;
mod union3;
mod intersection3;

use std::collections::HashMap;
use crate::{Model, Shape, get_curves_and_facets};
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
        let (curves, facets) = get_curves_and_facets(&self.parts);
        let mut curve_ranges: HashMap<usize, CurveRange> = HashMap::new(); 
        for (i, curve) in curves.iter().enumerate() {
            let params = curve.get_param_samples(4, cell_size);
            let step = curve.get_param_step(4, cell_size);
            curve_ranges.insert(i, CurveRange{i, step, params});
        }
        if facets.is_empty(){
            let mut basis = UnionBasis2 {
                intersections: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                curve_ranges,
                cell_size,
                shapes: vec![],
                tolerance: 0.05,
                max_walk_iterations: 1000,
                samples: vec![],
            };
            basis.get_shapes()
        }else{
            let mut facet_ranges: HashMap<usize, FacetRange> = HashMap::new(); 
            for (i, facet) in facets.iter().enumerate() {
                let (step, params) = facet.get_param_step_and_samples(4, cell_size);
                facet_ranges.insert(i, FacetRange{i, step, params});
            }
            let mut basis = UnionBasis3 {
                intersections: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                curve_ranges,
                cell_size,
                shapes: vec![],
                tolerance: 0.05,
                max_walk_iterations: 1000,
                samples: vec![],
            };
            basis.get_shapes()
        }
    }
}

#[derive(Clone)]
struct CurveRange {
    i: usize,
    step: f32,
    params: Vec<f32>,
}

#[derive(Clone)]
struct FacetRange {
    i: usize,
    step: Vec2,
    params: Vec<Vec2>,
}

