use glam::*;
use crate::log;
use crate::Shape;
use super::TestPair;

#[derive(Default, Debug)]
pub struct HoneTexels{
    pub shape: Vec<f32>,
    pub spreads: [Vec<Spread>; 3],
    // pub pairs: Vec<usize>,
    // pub index: Vec<i32>,
    // pub param: Vec<f32>,
}

#[derive(Default, Debug)]
pub struct Spread{
    pub pairs: Vec<usize>,
    pub index: Vec<i32>,
    pub param: Vec<f32>,
}

impl Spread {
    fn add_1(&mut self, pair: usize, index: [i32; 2], params: [f32; 4]) {
        self.pairs.push(pair);
        self.index.extend(index);
        self.param.extend(params);
    }   
}

pub fn hone_basis(shapes: &Vec<Shape>, pairs: &Vec<TestPair>) -> HoneTexels {
    let mut texels = HoneTexels::default(); 
    let mut spreads: [Vec<Spread>; 3] = [
        vec![Spread::default()], // not used
        (0..=2).map(|_| Spread::default()).collect(),
        (0..=2).map(|_| Spread::default()).collect()
    ];
    let mut indices = vec![];
    let mut knots = vec![];
    for shape in shapes {
        indices.push(texels.shape.len());
        texels.shape.extend(shape.texels());
        knots.push(shape.param_spread());
    }
    for (pi, TestPair{i0, i1, ..}) in pairs.iter().enumerate() {
        let ti = [indices[*i0] as i32, indices[*i1] as i32];
        let rank = |r0, r1| {shapes[*i0].rank == r0 && shapes[*i1].rank == r1};
        if rank(1, 0) {
            for u0 in &knots[*i0] {
                spreads[1][0].add_1(pi, ti, [u0[0], 0., 0., 0.]);
            }  
        } else if rank(1, 1) { 
            for u0 in &knots[*i0] {
                for u1 in &knots[*i1] {
                    spreads[1][1].add_1(pi, ti, [u0[0], u1[0], 0., 0.]);
                }  
            }  
        }
        // } else if rank(1, 2) { 
        //     for u0 in &knots[*i0] {
        //         for u1 in &knots[*i1] {
        //             spreads[1][2].add_1(pi, ti, [u0[0], u1[0], u1[1], 0.]);
        //         }  
        //     }  
        // } else if rank(2, 0) { 
        //     // for u0 in &knots[*i0] {
        //     //     spreads[2][0].add_1(pi, ti, [u0[0], u1[1], 0., 0.]);
        //     // }  
        // }
    }
    texels.spreads = spreads;
    texels
}





// pub fn hone_basis(shapes: &Vec<Shape>, pairs: &Vec<TestPair>) -> HoneBasis {
//     let mut basis = HoneBasis::default();
//     let mut param_texels1:   Vec<f32> = vec![];
//     let mut shape_indices: Vec<usize> = vec![];
//     for shape in shapes {
//         shape_indices.push(basis.shape_texels.len());
//         let section = add_section(shape, basis.max_knot_len as usize);
//         basis.shape_texels.extend(section.texels);
//         basis.max_knot_len = section.max_knot_len as i32;
//     }
//     for pair in pairs {
//         let ti0 = shape_indices[pair.i0];
//         let ti1 = shape_indices[pair.i1];
//         for params0 in shapes[pair.i0].param_spread() {
//             for params1 in shapes[pair.i1].param_spread() {
//                 basis.pairs.push(pair.clone());
//                 basis.pair_texels.extend([ti0 as i32, ti1 as i32]);
//                 for pi in 0..1 {
//                     if pi < params0.len() {
//                         basis.param_texels.push(params0[pi]);
//                     }else{
//                         basis.param_texels.push(-1.);
//                     }
//                 }
//                 for pi in 0..1 {
//                     if pi < params1.len() {
//                         param_texels1.push(params1[pi]);
//                     }else{
//                         param_texels1.push(-1.);
//                     }
//                 }
//             }  
//         }  
//     }
//     basis
// }



// pub struct Section {
//     texels:       Vec<f32>,
//     max_knot_len: usize,
// }

// fn add_section(shape: &Shape, max_knot_len0: usize) -> Section {
//     let mut max_knot_len = max_knot_len0;
//     if shape.basis.knots.len() > max_knot_len0 { 
//         max_knot_len = shape.basis.knots.len(); 
//     }
//     let mut texels = shape.texels();
//     let mut control_indices = vec![];
//     let mut control_texels = vec![];
//     for control in &shape.controls {
//         let control_index = texels.len() + shape.controls.len() + control_texels.len();
//         control_indices.push(control_index as f32);
//         let section = add_section(control, max_knot_len);
//         control_texels.extend(section.texels);
//         max_knot_len = section.max_knot_len;
//     }
//     texels.extend(control_indices);
//     texels.extend(control_texels);
//     Section{texels, max_knot_len}
// }

// fn shape_texels(shape: &Shape) {
//     let mut texels: Vec<f32> = vec![];
//     texels.extend([
//         shape.rank as f32, 
//         shape.controls.len() as f32,
//         shape.basis.order as f32,
//         shape.basis.min,
//         shape.basis.max,
//     ]); 
//     texels.extend(&shape.basis.knots);
//     texels.extend(&shape.basis.weights);
//     texels
// }