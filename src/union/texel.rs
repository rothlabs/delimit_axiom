use glam::{vec2, Vec2};
use crate::FacetShape;

struct IndexedUV {
    facet_i: usize,
    texel_i:   usize,
    uv: Vec2
}

#[derive(Clone)]
pub struct IndexPair {
    pub g0: usize,
    pub g1: usize,
    pub f0: usize,
    pub f1: usize,
}

#[derive(Default)]
pub struct TexelBasis{
    pub index_pairs: Vec<IndexPair>,
    pub pair_texels: Vec<i32>,
    pub facet_texels: Vec<f32>,
    pub uv_texels: Vec<f32>,
    pub max_facet_length: i32,
    pub max_knot_count: i32,
}

impl TexelBasis {
    pub fn new(groups: Vec<Vec<FacetShape>>) -> Self{
        let mut max_facet_length = 0;
        let mut max_knot_count = 0;
        let mut index_pairs: Vec<IndexPair> = vec![];
        let mut indexed_uv_groups: Vec<Vec<IndexedUV>> = vec![];
        let mut facet_texels: Vec<f32> = vec![];
        let mut pair_texels: Vec<i32> = vec![];
        let mut uv_texels: Vec<f32> = vec![];
        for group in groups {
            let mut indexed_uvs: Vec<IndexedUV> = vec![];
            for (facet_i, facet) in group.iter().enumerate() {
                if facet.nurbs.knots.len() > max_knot_count {
                    max_knot_count = facet.nurbs.knots.len();
                }
                let texel_i = facet_texels.len();
                facet_texels.extend([
                    10000000.,
                    facet.controls.len() as f32,
                    facet.nurbs.order as f32,
                ]);
                facet_texels.extend(&facet.nurbs.knots);
                facet_texels.extend(&facet.nurbs.weights);
                for (ci, curve) in facet.controls.iter().enumerate() {
                    if curve.nurbs.knots.len() > max_knot_count { 
                        max_knot_count = curve.nurbs.knots.len(); 
                    }
                    facet_texels.extend([
                        9000000. + ci as f32,
                        curve.controls.len() as f32,
                        curve.nurbs.order as f32,
                        curve.min,
                        curve.max,
                    ]); 
                    for i in 0..curve.nurbs.knots.len()-1 {
                        if curve.nurbs.knots[i] < curve.nurbs.knots[i+1] || i == curve.nurbs.knots.len() - curve.nurbs.order {
                            indexed_uvs.push(IndexedUV{
                                facet_i, texel_i, uv:vec2(curve.nurbs.knots[i], ci as f32 / (facet.controls.len()-1) as f32)}); 
                        }
                        facet_texels.push(curve.nurbs.knots[i]);
                    }  
                    facet_texels.push(curve.nurbs.knots[curve.nurbs.knots.len()-1]);
                    facet_texels.extend(&curve.nurbs.weights);
                    for point in &curve.controls {
                        facet_texels.extend(point.to_array());
                    }
                }
                let facet_length = facet_texels.len() - texel_i;
                if facet_length > max_facet_length{
                    max_facet_length = facet_length;
                } 
            }
            indexed_uv_groups.push(indexed_uvs);
        }
        for g1 in 1..indexed_uv_groups.len(){
            for g0 in 0..g1{
                for IndexedUV{facet_i:f0, texel_i:t0, uv:uv0} in &indexed_uv_groups[g0]{
                    for IndexedUV{facet_i:f1, texel_i:t1, uv:uv1} in &indexed_uv_groups[g1]{
                        index_pairs.push(IndexPair{g0, g1, f0:*f0, f1:*f1});
                        pair_texels.push(*t0 as i32);
                        pair_texels.push(*t1 as i32);
                        uv_texels.extend(uv0.to_array());
                        uv_texels.extend(uv1.to_array());
                    }  
                }   
            }
        }
        TexelBasis {
            index_pairs,
            pair_texels,
            facet_texels,
            uv_texels,
            max_facet_length: max_facet_length as i32,
            max_knot_count: max_knot_count as i32,
        }
    }
}

#[derive(Default)]
pub struct TraceTexelBasis{
    pub index_pairs: Vec<IndexPair>,
    pub pair_texels: Vec<i32>,
    pub uv_box_texels:   Vec<Vec<f32>>,
}

impl TraceTexelBasis {
    pub fn new(basis: &TexelBasis, hit_miss: Vec<f32>) -> Self {
        let mut index_pairs: Vec<IndexPair> = vec![];
        let mut pair_texels: Vec<i32> = vec![];
        let mut uv_texels:   Vec<f32> = vec![];
        let mut box_texels:  Vec<f32> = vec![];
        for k in 0..2 {
            for i in 0..basis.index_pairs.len() {
                if hit_miss[i*4] > -0.5 { // it's a hit
                    if k < 1 {
                        index_pairs.push(basis.index_pairs[i].clone());
                    }
                    pair_texels.extend([basis.pair_texels[i*2+0], basis.pair_texels[i*2+1]]);
                    uv_texels.extend([hit_miss[i*4+0], hit_miss[i*4+1], hit_miss[i*4+2], hit_miss[i*4+3]]); // use .slice of uv_pairs
                    box_texels.extend([1., 1., 0., 0.]);
                    // hit_points.push({
                    //     ...group_facet_indices0[i],
                    //     uv0: [hit_miss[i*4+0], hit_miss[i*4+1]],
                    //     uv1: [hit_miss[i*4+2], hit_miss[i*4+3]],
                    // });
                }
            }
        }
        TraceTexelBasis {
            index_pairs, 
            pair_texels,
            uv_box_texels: vec![uv_texels, box_texels],
        }
    }
}