use glam::*;
use crate::{CurveShape, FacetShape, Shape, HitBasis3, Trim};
use super::union2::UnionBasis2;

pub struct UnionBasis3 {
    pub hit_basis: HitBasis3,
    pub curve_groups: Vec<Vec<CurveShape>>,
    pub facet_groups: Vec<Vec<FacetShape>>,
    pub shapes: Vec<Shape>,
}

impl UnionBasis3 { 
    pub fn get_shapes(
        curve_groups: Vec<Vec<CurveShape>>, facet_groups: Vec<Vec<FacetShape>>,
    ) -> Vec<Shape> {
        UnionBasis3 {
            hit_basis: HitBasis3::new(facet_groups.clone()),
            curve_groups,
            facet_groups,
            shapes: vec![],
        }.make_shapes()
    }

    pub fn make_shapes(&mut self) -> Vec<Shape> {//-> (Vec<CurveShape>, Vec<FacetShape>) {
        self.hit_basis.make().expect("Facet intersection should succeed for union3 to work.");
        let hits = self.hit_basis.facet_hits.clone();
        let mut misses = self.hit_basis.facet_miss.clone();
        self.shapes = self.hit_basis.shapes.clone();
        for gi in 0..self.facet_groups.len() {
            for fi in 0..self.facet_groups[gi].len() {
                let mut collect_facet = false;
                for hi in 0..hits[gi][fi].len() {
                    if hits[gi][fi][hi].is_empty() {
                        misses[gi][fi][hi].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                        if misses[gi][fi][hi].is_empty() || misses[gi][fi][hi][0].dot * self.facet_groups[gi][fi].nurbs.sign < 0. {   
                            collect_facet = true;
                        }else{
                            collect_facet = false;
                            break; // This should ensure the facet is not collected in later hit groups
                        }
                    }else{
                        self.union_facet_with_hits(gi, fi, hi);  
                        collect_facet = true;
                    }
                }
                if collect_facet {
                    let mut facet = self.facet_groups[gi][fi].clone();
                    if facet.nurbs.sign < 0. {facet.reverse().negate();}
                    self.shapes.push(Shape::Facet(facet));
                }
            }
        }
        for curve_group in &self.curve_groups {
            for curve in curve_group {
                self.shapes.push(Shape::Curve(curve.clone()));
            }
        }
        self.shapes.clone()
    }

    fn union_facet_with_hits(&mut self, gi: usize, fi: usize, hi: usize) {
        let facet = self.facet_groups[gi].get_mut(fi).expect("Should be a facet at this index.");
        if facet.nurbs.sign < 0. {
            for curve in &mut facet.boundaries {
                curve.negate();
            }
        }
        // for j in 0..facet.boundaries.len() {
        //     let mut bndry = facet.boundaries[j].clone();
        //     bndry.controls.clear();
        //     for k in 0..facet.boundaries[j].controls.len() {
        //         bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
        //             100. + fi as f32 * 2.,// + (j as f32)*0.005,  
        //             gi as f32 * 2.,// + (j as f32)*0.01, 
        //             0.
        //         ));
        //     }
        //     self.shapes.push(Shape::Curve(bndry));
        // }
        // for j in 0..self.hit_basis.facet_hits[gi][fi][hi].len() {
        //     let mut bndry = self.hit_basis.facet_hits[gi][fi][hi][j].clone();
        //     bndry.controls.clear();
        //     for k in 0..self.hit_basis.facet_hits[gi][fi][hi][j].controls.len() {
        //         bndry.controls.push(self.hit_basis.facet_hits[gi][fi][hi][j].controls[k] + vec3(
        //             100. + fi as f32 * 2.,// + (j as f32)*0.01,  
        //             gi as f32 * 2.,//  + (j as f32)*0.01, 
        //             0.
        //         ));
        //     }
        //     self.shapes.push(Shape::Curve(bndry));
        // }
        let mut trim = Trim::new(self.hit_basis.facet_hits[gi][fi][hi].clone()); // 0.001
        let curves1 = trim.build();
        // for j in 0..curves1.len() {
        //     let mut bndry = curves1[j].clone();
        //     bndry.controls.clear();
        //     for k in 0..curves1[j].controls.len() {
        //         bndry.controls.push(curves1[j].controls[k] + vec3(
        //             100. + fi as f32 * 2., //  + (j as f32)*0.01  
        //             gi as f32 * 2., //  + (j as f32)*0.01 
        //             0.
        //         ));
        //     }
        //     self.shapes.push(Shape::Curve(bndry));
        // }
        let mut union = UnionBasis2::new(facet.boundaries.clone(), curves1.clone()); // self.facet_hits[g][i].clone()
        facet.boundaries = union.build();
        for j in 0..facet.boundaries.len() {
            let mut bndry = facet.boundaries[j].clone();
            bndry.controls.clear();
            for k in 0..facet.boundaries[j].controls.len() {
                bndry.controls.push(facet.boundaries[j].controls[k] + vec3(
                    100. + fi as f32 * 2.,// + (j as f32)*0.005,  
                    gi as f32 * 2.,// + (j as f32)*0.01, 
                    0.
                ));
            }
            self.shapes.push(Shape::Curve(bndry));
        }
    }
}