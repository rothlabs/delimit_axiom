mod union2;
mod union3;

use crate::{get_grouped_curves_and_facets, Group, HitTester2, HitTester3, Model, Shape, Spatial3};
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
        let tolerance = 0.005;
        let duplication_tolerance = tolerance * 10.;
        let hit_step = 2.;
        let (curves, facets, grouped_curves, grouped_facets) = get_grouped_curves_and_facets(&self.parts);
        if facets.is_empty(){
            let mut basis = UnionBasis2 {
                tester: HitTester2 {
                    curves: curves.clone(),
                    index0: 0,
                    index1: 0,
                    spatial: Spatial3::new(duplication_tolerance), // (0..curves.len()).map(|_| Spatial3::new(duplication_tolerance)).collect(),
                    points:  vec![], //(0..curves.len()).map(|_| vec![]).collect(),
                    normal: Vec3::Z,
                    tolerance,
                    duplication_tolerance,
                },
                hits: (0..curves.len()).map(|_| vec![]).collect(),
                miss: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                groups: grouped_curves,
                shapes: vec![],
            };
            basis.get_shapes()
        }else{
            let mut basis = UnionBasis3 {
                tester: HitTester3 {
                    curves: curves.clone(),
                    facets: facets.clone(),
                    facet_index0: 0,
                    facet_index1: 0,
                    hit_step,
                    hit_map:    (0..facets.len()).map(|_| Spatial3::new(hit_step)).collect(),
                    hit_points: (0..facets.len()).map(|_| vec![]).collect(),
                    tolerance: 0.05,
                },
                facet_hits: (0..curves.len()).map(|_| vec![]).collect(),
                //curve_hits: (0..curves.len()).map(|_| vec![]).collect(),
                curves,
                facets,
                grouped_curves,
                grouped_facets,
                shapes: vec![],
            };
            self.transform.get_reshapes(basis.get_shapes())
        }
    }
}


//let seed: [u8; 32] = *b"01234567891234560123456789123456";
//rng: StdRng::from_seed(seed),


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