use std::{collections::HashMap, f32::EPSILON};
use crate::{CurveShape, Shape, Spatial3};
use super::{intersection3::Intersection3, CurveParams, FacetParams};
use glam::*;


pub struct Sample3 {
    curve: usize,
    point: Vec3,
    u: f32,
}

//#[derive(Clone, Default)]
pub struct UnionBasis3 {
    pub curves: Vec<CurveShape>,
    pub curve_ranges: HashMap<usize, CurveParams>, 
    pub facet_ranges: HashMap<usize, FacetParams>, 
    pub cell_size: f32,
    pub shapes: Vec<Shape>,
    pub intersections: Vec<Vec<Intersection3>>,
    pub tolerance: f32,
    pub max_walk_iterations: usize,
    pub samples: Vec<Sample3>,
}

impl UnionBasis3 { 
    pub fn get_shapes(&mut self) -> Vec<Shape> {
        self.reduce_ranges(false);
        self.reduce_ranges(true);
        for i in 0..self.curves.len() {
            let mut curve = self.curves[i].clone();
            self.intersections[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
            if self.intersections[i].is_empty() {
                self.shapes.push(Shape::Curve(curve));
               continue;
            }
            let first = self.intersections[i].first().unwrap();
            let mut set_min = false;
            if first.dot > 0. {set_min = true;}
            let mut point = first.point;
            let mut intersections = vec![first];
            for itc in &self.intersections[i] {
                if itc.point.distance(point) > self.cell_size {
                    intersections.push(itc);
                }
                point = itc.point;
            }
            for itc in intersections { 
                self.shapes.push(Shape::Point(vec3(itc.point.x, itc.point.y, 0.)));
                if set_min {
                    curve.min = itc.u;
                }else{
                    curve.max = itc.u;
                    self.shapes.push(Shape::Curve(curve));
                    curve = self.curves[i].clone();
                }
                set_min = !set_min;
            }
            if !set_min {
                self.shapes.push(Shape::Curve(curve));
            }   
        }
        self.shapes.clone()
    }

    fn reduce_ranges(&mut self, add_intersections: bool) { 
        let spatial_map = self.get_spatial_map();
        for i in 0..self.curves.len() {
            if let Some(cr) = self.curve_ranges.get_mut(&i) {
                cr.params.clear();
            }
        }
        spatial_map.for_pairs(&mut |i0: usize, i1: usize| { //for pairs in spatial_map.get_pairs() {
            let Sample3 {curve: c0, point: p0, u: u0} = self.samples[i0];
            let Sample3 {curve: c1, point: p1, u: u1} = self.samples[i1];
            if c0 == c1 {return}
            if p0.distance(p1) > self.cell_size {return}
            if let Some(cr) = self.curve_ranges.get_mut(&c0) {
                cr.params.push(u0);
            }
            if add_intersections {
                if let Some(itc) = self.get_intersection(&c0, &c1, u0, u1) {
                    if 0.01 < itc.u && itc.u < 0.99 {
                        self.intersections[c0].push(itc.clone());
                    } 
                }
            }
        });
        for i in 0..self.curves.len() {
            if let Some(cr) = self.curve_ranges.get_mut(&i) {
                cr.params.sort_by(|a, b| a.partial_cmp(b).unwrap());
                if cr.params.is_empty() {continue}
                let mut filled = vec![cr.params[0]];
                for uu in cr.params.windows(2) {
                    if uu[1] - uu[0] <= cr.step + EPSILON {
                        for k in 1..20 {
                            let fill_u = uu[0] + k as f32 * (cr.step/10.);
                            if fill_u >= uu[1] {break}
                            filled.push(fill_u);
                        }
                    }
                    filled.push(uu[1]);
                }
                cr.params = filled;
                cr.step /= 10.
            }
        }
        self.cell_size /= 10.;
    }

    fn get_spatial_map(&mut self) -> Spatial3 { 
        let mut spatial_map: Spatial3 = Spatial3::new(self.cell_size); 
        for (_, CurveParams {i, params, ..}) in &self.curve_ranges { 
            for u in params {
                let point = self.curves[*i].get_vec3_at_u(*u);
                self.samples.push(Sample3 {
                    curve: *i,
                    point,
                    u: *u,
                });
                spatial_map.insert(&point, self.samples.len()-1);
            }
        }
        spatial_map
    }
}