use std::{collections::HashMap, f32::EPSILON};
use crate::{nurbs::curve, CurveShape, Shape, Spatial2};
use super::{intersection2::Intersection2, CurveParams};
use glam::*;


pub struct Sample2 {
    curve: usize,
    point: Vec2,
    u: f32,
}

pub struct UnionBasis2 {
    pub curves: Vec<CurveShape>,
    pub curve_ranges: HashMap<usize, CurveParams>, 
    pub cell_size: f32,
    pub shapes: Vec<Shape>,
    pub intersections: Vec<Vec<Intersection2>>,
    pub tolerance: f32,
    pub max_walk_iterations: usize,
    pub samples: Vec<Sample2>,
}

impl UnionBasis2 { 
    pub fn get_shapes(&mut self) -> Vec<Shape> {
        let spatial = self.set_samples_and_get_spatial();
        self.clear_params();
        self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_curve_param);
        self.reduce_cell_and_step();
        let spatial = self.set_samples_and_get_spatial();
        self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_intersection);
        for i in 0..self.curves.len() {
            self.intersections[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
            if self.intersections[i].is_empty() {
                self.shapes.push(Shape::Curve(self.curves[i].clone()));
                continue;
            }
            self.add_split_curves(i);
        }
        self.shapes.clone()
    }

    fn add_split_curves(&mut self, i: usize) {
        let first = self.intersections[i].first().unwrap();
        let mut set_min = false;
        if first.angle > 0. {set_min = true;}
        let mut curve = self.curves[i].clone();
        for itc in self.get_merged_intersections(i, first) { 
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

    fn get_merged_intersections(&self, i: usize, first: &Intersection2) -> Vec<Intersection2> {
        let mut point = first.point;
        let mut intersections = vec![first.clone()];
        for itc in &self.intersections[i] {
            if itc.point.distance(point) > self.cell_size {
                intersections.push(itc.clone());
            }
            point = itc.point;
        }
        intersections
    }

    fn clear_params(&mut self) {
        for i in 0..self.curves.len() {
            if let Some(cr) = self.curve_ranges.get_mut(&i) {
                cr.params.clear();
            }
        }
    }

    fn add_curve_param(&mut self, curve_index0: usize, _c1: usize, u0: f32, _u1: f32) {
        if let Some(cr) = self.curve_ranges.get_mut(&curve_index0) {
            cr.params.push(u0);
        }
    }

    fn add_intersection(&mut self, curve_index0: usize, curve_index1: usize, u0: f32, u1: f32) {
        if let Some(itc) = self.get_intersection(&curve_index0, &curve_index1, u0, u1) {
            if 0.01 < itc.u && itc.u < 0.99 {
                self.intersections[curve_index0].push(itc.clone());
            } 
        }
    }

    fn for_spatial_pairs<F>(&mut self, spatial: &Spatial2, func: &mut F) 
    where F: FnMut(&mut UnionBasis2, usize, usize, f32, f32)  { 
        spatial.for_pairs(&mut |i0: usize, i1: usize| { 
            let Sample2 {curve: c0, point: p0, u: u0} = self.samples[i0];
            let Sample2 {curve: c1, point: p1, u: u1} = self.samples[i1];
            if c0 == c1 {return}
            if p0.distance(p1) > self.cell_size {return}
            func(self, c0, c1, u0, u1);
        });
    }

    fn reduce_cell_and_step(&mut self) {
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

    fn set_samples_and_get_spatial(&mut self) -> Spatial2 { 
        let mut spatial: Spatial2 = Spatial2::new(self.cell_size); 
        self.samples.clear();
        for (_, CurveParams {i, params, ..}) in &self.curve_ranges { 
            for u in params {
                let point = self.curves[*i].get_vec2_at_u(*u);
                self.samples.push(Sample2 {
                    curve: *i,
                    point,
                    u: *u,
                });
                spatial.insert(&point, self.samples.len()-1);
            }
        }
        spatial
    }
}


//for_merged_intersections(&self.intersections[i].clone(), self.cell_size, &mut |itc: &Intersection2| { 

// fn for_merged_intersections<F>(intersections: &Vec<Intersection2>, tolerance: f32, func: &mut F)// -> Vec<Intersection2> 
// where F: FnMut(&Intersection2)  {
//     let mut point = intersections.first().unwrap().point;
//     //let mut intersections = vec![first.clone()];
//     for itc in intersections{
//         if itc.point.distance(point) > tolerance {
//             func(&itc);
//             //intersections.push(itc.clone());
//         }
//         point = itc.point;
//     }
//     //intersections
// }