use glam::{vec2, Vec2};
use crate::FacetShape;

struct IndexedUV {
    facet_i: usize,
    texel_i:   usize,
    uv: Vec2
}

pub struct IndexPair {
    g0: usize,
    g1: usize,
    f0: usize,
    f1: usize,
}

pub struct HonePrep{
    pub index_pairs: Vec<IndexPair>,
    pub pair_texels: Vec<i32>,
    pub facet_texels: Vec<f32>,
    pub uv_texels: Vec<f32>,
    pub max_facet_length: usize,
    pub max_knots: usize,
}

pub fn get_hone_prep(groups: Vec<Vec<FacetShape>>) -> HonePrep{
    let mut max_facet_texels = 0;
    let mut max_knots = 0;
    let mut index_pairs: Vec<IndexPair> = vec![];
    let mut indexed_uv_groups: Vec<Vec<IndexedUV>> = vec![];
    let mut facet_tex: Vec<f32> = vec![];
    let mut pair_tex: Vec<i32> = vec![];
    let mut uv_tex: Vec<f32> = vec![];
    for group in groups {
        let mut indexed_uvs: Vec<IndexedUV> = vec![];
        for (fi, facet) in group.iter().enumerate() {
            if facet.nurbs.knots.len() > max_knots {
                max_knots = facet.nurbs.knots.len();
            }
            let texel_i = facet_tex.len();
            facet_tex.extend([
                10000000.,
                facet.controls.len() as f32,
                facet.nurbs.order as f32,
            ]);
            facet_tex.extend(&facet.nurbs.knots);
            facet_tex.extend(&facet.nurbs.weights);
            for (ci, curve) in facet.controls.iter().enumerate() {
                if curve.nurbs.knots.len() > max_knots { 
                    max_knots = curve.nurbs.knots.len(); 
                }
                facet_tex.extend([
                    9000000. + ci as f32,
                    curve.controls.len() as f32,
                    curve.nurbs.order as f32,
                    curve.min,
                    curve.max,
                ]); 
                for i in 0..curve.nurbs.knots.len()-1 {
                    if curve.nurbs.knots[i] < curve.nurbs.knots[i+1] || i == curve.nurbs.knots.len() - curve.nurbs.order {
                        indexed_uvs.push(IndexedUV{
                            facet_i:fi, texel_i, uv:vec2(curve.nurbs.knots[i], ci as f32 / (facet.controls.len()-1) as f32)}); 
                    }
                    facet_tex.push(curve.nurbs.knots[i]);
                }  
                facet_tex.push(curve.nurbs.knots[curve.nurbs.knots.len()-1]);
                facet_tex.extend(&curve.nurbs.weights);
                for point in &curve.controls {
                    facet_tex.extend(point.to_array());
                }
            }
            let facet_length = facet_tex.len() - texel_i;
            if facet_length > max_facet_texels{
                max_facet_texels = facet_length;
            } 
        }
        indexed_uv_groups.push(indexed_uvs);
    }
    for g1 in 1..indexed_uv_groups.len(){
        for g0 in 0..g1{
            for IndexedUV{facet_i:f0, texel_i:t0, uv:uv0} in &indexed_uv_groups[g0]{
                for IndexedUV{facet_i:f1, texel_i:t1, uv:uv1} in &indexed_uv_groups[g1]{
                    index_pairs.push(IndexPair{g0, g1, f0:*f0, f1:*f1});
                    pair_tex.push(*t0 as i32);
                    pair_tex.push(*t1 as i32);
                    uv_tex.extend(uv0.to_array());
                    uv_tex.extend(uv1.to_array());
                }  
            }   
        }
    }
    HonePrep {
        index_pairs,
        pair_texels: pair_tex,
        facet_texels: facet_tex,
        uv_texels: uv_tex,
        max_facet_length: max_facet_texels,
        max_knots,
    }
}