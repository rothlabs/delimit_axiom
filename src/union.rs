use std::{collections::HashMap, f32::EPSILON};
use crate::{get_curves, log, CurveShape, Model, Shape, SpatialMap};
use serde::{Deserialize, Serialize};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Union::default")]
pub struct Union {
    pub parts: Vec<Model>,
}

impl Union {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let cell_size = 4.;
        let curves = get_curves(&self.parts);
        let mut curve_ranges: HashMap<usize, CurveRange> = HashMap::new(); // = vec![];
        for (i, curve) in curves.iter().enumerate() {
            let params = curve.get_param_samples(4, cell_size);
            let step = curve.get_param_step(4, cell_size);
            curve_ranges.insert(i, CurveRange{
                i, 
                step,
                params,
            });
        }
        let mut union_shape = UnionShape {
            intersections: (0..curves.len()).map(|_| vec![]).collect(),//intersection_map: SpatialMap::new(0.025),
            curves,
            curve_ranges,
            cell_size,
            shapes: vec![],
            tolerance: 0.01,
            max_walk_iterations: 1000,
            samples: vec![],
        };
        union_shape.get_shapes()
    }
}

#[derive(Clone, Default)]
struct Sample {
    curve: usize,
    point: Vec2,
    u: f32,
}

#[derive(Clone, Default)]
struct CurveRange {
    i: usize,
    step: f32,
    params: Vec<f32>,
}

//#[derive(Clone, Default)]
struct UnionShape {
    curves: Vec<CurveShape>,
    curve_ranges: HashMap<usize, CurveRange>, 
    cell_size: f32,
    shapes: Vec<Shape>,
    //intersection_map: SpatialMap,
    intersections: Vec<Vec<Intersection>>,
    tolerance: f32,
    max_walk_iterations: usize,
    samples: Vec<Sample>,
}

#[derive(Clone)]
struct Intersection {
    u: f32,
    angle: f32,
    point: Vec2,
}

impl UnionShape { 
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
            if first.angle > 0. {set_min = true;}
            let mut point = first.point;
            let mut intersections = vec![first];
            for itc in &self.intersections[i] {
                if itc.point.distance(point) > self.cell_size*2. {
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
            let Sample {curve: c0, point: p0, u: u0} = self.samples[i0];
            let Sample {curve: c1, point: p1, u: u1} = self.samples[i1];
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

    fn get_spatial_map(&mut self) -> SpatialMap { 
        let mut spatial_map: SpatialMap = SpatialMap::new(self.cell_size); 
        for (_, CurveRange {i, params, ..}) in &self.curve_ranges { 
            for u in params {
                let point = self.curves[*i].get_vec2_at_u(*u);
                self.samples.push(Sample {
                    curve: *i,
                    point,
                    u: *u,
                });
                spatial_map.insert(&point, self.samples.len()-1);
            }
        }
        spatial_map
    }

    fn get_intersection(&self, c0: &usize, c1: &usize, u_start0: f32, u_start1: f32) -> Option<Intersection> {
        let curve0 = &self.curves[*c0];
        let curve1 = &self.curves[*c1];
        let mut move_t0 = true; 
        let mut u0 = u_start0;
        let mut u1 = u_start1;
        let mut p0 = curve0.get_vec2_at_u(u0);
        let mut p1 = curve1.get_vec2_at_u(u1);
        let mut distance = p0.distance(p1);
        let mut dir0 = curve0.get_param_step(4, self.cell_size/10.);
        let mut dir1 = curve1.get_param_step(4, self.cell_size/10.); 
        for i in 0..self.max_walk_iterations {
            if distance < self.tolerance { 
                break; 
            }
            // if i == self.max_walk_iterations-1 {
            //     log("Hit max iterations in get_intersection!");
            // }
            if move_t0 {
                u0 = (u0 + dir0).clamp(0., 1.);
                p0 = curve0.get_vec2_at_u(u0);
            }else{
                u1 = (u1 + dir1).clamp(0., 1.);
                p1 = curve1.get_vec2_at_u(u1);
            }
            let dist = p0.distance(p1);
            if dist >= distance {
                if move_t0 {
                    dir0 = dir0 * -0.99;
                }else{
                    dir1 = dir1 * -0.99;
                }
                move_t0 = !move_t0;
            }
            distance = dist;
        }
        if distance < self.tolerance {
            let delta = 0.0001;
            let d0 = u0 + delta;
            let d1 = u1 + delta;
            let pd0 = curve0.get_vec2_at_u(d0);
            let pd1 = curve1.get_vec2_at_u(d1);
            if let Some(ip) = get_line_intersection(p0, pd0, p1, pd1) {
                let ratio = p0.distance(ip) / p0.distance(pd0);
                let mut u = u0 + (d0-u0)*ratio;
                let mut point = curve0.get_vec2_at_u(u);
                let alt_u = u0 + (u0-d0)*ratio;
                let alt_point = curve0.get_vec2_at_u(alt_u);
                if alt_point.distance(ip) < point.distance(ip) {
                    u = alt_u;
                    point = alt_point;
                }
                let angle = (pd0-p0).angle_between(pd1-p1);
                Some(Intersection {
                    u,
                    point,
                    angle,
                })
            }else{
                None
            }
        }else{
            None
        }
    }
}

fn get_line_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    // let t = ((p1.x - p3.x)*(p3.y - p4.y) - (p1.y - p3.y)*(p3.x - p4.x)) 
    //     / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    // let x = p1.x + t*(p2.x - p1.x);
    // let y = p1.y + t*(p2.y - p1.y);
    let u = - ((p1.x - p2.x)*(p1.y - p3.y) - (p1.y - p2.y)*(p1.x - p3.x))
        / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    let x = p3.x + u*(p4.x - p3.x);
    let y = p3.y + u*(p4.y - p3.y);
    if x.is_nan() || y.is_nan() {
        return None;
    }
    Some(vec2(x, y))
}