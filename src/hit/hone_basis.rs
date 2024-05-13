use glam::*;
use crate::log;
use crate::Shape;
use super::TestPair;

#[derive(Default, Debug)]
pub struct HoneBasis{
    pub pairs:          Vec<TestPair>,
    pub pair_texels:    Vec<i32>,
    pub shape_texels:   Vec<f32>,
    pub param_texels:   Vec<f32>,
}

pub fn hone_basis(shapes: &Vec<Shape>, pairs: &Vec<TestPair>) -> HoneBasis {
    let mut basis = HoneBasis::default();
    let mut indices: Vec<usize> = vec![];
    for shape in shapes {
        indices.push(basis.shape_texels.len());
        basis.shape_texels.extend(shape.texels());
    }
    for pair in pairs {
        if shapes[pair.i0].rank != 1 || shapes[pair.i1].rank != 1 {
            continue;
        }
        let ti0 = indices[pair.i0];
        let ti1 = indices[pair.i1];
        for params0 in shapes[pair.i0].param_spread() {
            for params1 in shapes[pair.i1].param_spread() {
                basis.pairs.push(pair.clone());
                basis.pair_texels.extend([ti0 as i32, ti1 as i32]);
                basis.param_texels.extend([params0[0], params1[0], 0., 0.]);
            }  
        }  
    }
    basis
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