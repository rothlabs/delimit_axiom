use std::{collections::HashMap, f32::{consts::PI, EPSILON, INFINITY, NEG_INFINITY}, fmt::Result};
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
        //let mut curve_steps = vec![];
        
        for (i, curve) in curves.iter().enumerate() {
            let params = curve.get_param_samples(4, cell_size);
            let step = curve.get_param_step(4, cell_size);
            curve_ranges.insert(i, CurveRange{
                i, 
                step,
                params,
            });
            //curve_steps.push(curve.get_param_step(4, cell_size/2.));
        }
        let mut union_shape = UnionShape {
            intersections: (0..curves.len()).map(|_| vec![]).collect(),//intersection_map: SpatialMap::new(0.025),
            curves,
            curve_ranges,
            //curve_steps,
            cell_size,
            shapes: vec![],
            tolerance: 0.01,
            max_walk_iterations: 400,
        };
        union_shape.get_shapes()
    }
}

#[derive(Clone, Default)]
struct SampleCell {
    //curves: Vec<CurveSample>,
    curves: Vec<usize>,
    points: Vec<Vec2>,
    params: Vec<f32>,
}

#[derive(Clone, Default)]
struct CurveRange {
    i: usize,//curve: Vec<CurveShape>,
    step: f32,
    params: Vec<f32>,
    // min: f32,
    // max: f32,
    //keep: bool,
}

//#[derive(Clone, Default)]
struct UnionShape {
    curves: Vec<CurveShape>,
    curve_ranges: HashMap<usize, CurveRange>, //Vec<CurveRange>,
    //curve_steps: Vec<f32>,
    cell_size: f32,
    shapes: Vec<Shape>,
    //intersection_map: SpatialMap<()>,
    intersections: Vec<Vec<Intersection>>,
    tolerance: f32,
    max_walk_iterations: usize,
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
                if itc.point.distance(point) > self.tolerance * 10. {
                    intersections.push(itc);
                    point = itc.point;
                }
            }
            for itc in intersections { 
                self.shapes.push(Shape::Point([itc.point.x, itc.point.y, 0.]));
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
                cr.params = vec![];
            }
        }
        let mut key_parts = [0; 2];
        for (cell_key, sample_cell0) in &spatial_map.map {
            let split_key = cell_key.split(",");
            for (i, string_int) in split_key.enumerate() {
                key_parts[i] = string_int.parse().expect("failed to parse key in union");
            }
            for (i0, c0) in sample_cell0.curves.iter().enumerate() {
                for x in -1..2 {
                    for y in -1..2 {
                        let key = (key_parts[0]+x).to_string()+","+&(key_parts[1]+y).to_string();// + ",";
                        let sample_cell1 = spatial_map.map.get(&key); 
                        if let Some(sample_cell1) = sample_cell1 {
                            for (i1, c1) in sample_cell1.curves.iter().enumerate() {
                                if c0 == c1 {continue}
                                if sample_cell0.points[i0].distance(sample_cell1.points[i1]) > self.cell_size {continue}
                                if let Some(cr) = self.curve_ranges.get_mut(&c0) {
                                    cr.params.push(sample_cell0.params[i0]);
                                }
                                if add_intersections {
                                    let itc = self.search_intersection(c0, c1, sample_cell0.params[i0], sample_cell1.params[i1]);
                                    if let Some(itc) = itc {
                                        if 0.01 < itc.u && itc.u < 0.99 {
                                            self.intersections[*c0].push(itc.clone());
                                            //self.shapes.push(Shape::Point([itc.point.x, itc.point.y, 0.]));
                                        } 
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
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

    fn get_spatial_map(&self) -> SpatialMap<SampleCell> { // , curves: Vec<CurveShape>, curve_ranges: Vec<CurveRange>, cell_size: f32
        let meta = String::from("");
        let mut spatial_map: SpatialMap<SampleCell> = SpatialMap::new(self.cell_size); 
        for (_, CurveRange {i, params, ..}) in &self.curve_ranges { 
            for u in params {
                //let u = min + step * step_i as f32;
                let point = self.curves[*i].get_vec2_at_u(*u);
                if let Some(sample_cell) = spatial_map.get_mut(&point, &meta) {
                    sample_cell.curves.push(*i);
                    sample_cell.points.push(point);
                    sample_cell.params.push(*u);
                }else{
                    spatial_map.insert(&point, &meta, &SampleCell {
                        curves: vec![*i],
                        points: vec![point],
                        params: vec![*u],
                    });
                }
            }
        }
        spatial_map
    }

    fn search_intersection(&self, c0: &usize, c1: &usize, u_start0: f32, u_start1: f32) -> Option<Intersection> {
        let curve0 = &self.curves[*c0];
        let curve1 = &self.curves[*c1];
        let mut move_t0 = true; 
        let mut t0 = u_start0;
        let mut t1 = u_start1;
        let mut p0 = curve0.get_vec2_at_u(t0);
        let mut p1 = curve1.get_vec2_at_u(t1);
        let mut distance = p0.distance(p1);
        let mut dir0 = curve0.get_param_step(4, self.cell_size/2.);
        let mut dir1 = curve1.get_param_step(4, self.cell_size/2.); 
        for i in 0..self.max_walk_iterations {
            if distance < self.tolerance { 
                break; 
            }
            // if i == self.max_walk_iterations-1 {
            //     //log("Hit max iterations in search_intersection!");
            // }
            if move_t0 {
                t0 = (t0 + dir0).clamp(0., 1.);
                p0 = curve0.get_vec2_at_u(t0);
            }else{
                t1 = (t1 + dir1).clamp(0., 1.);
                p1 = curve1.get_vec2_at_u(t1);
            }
            let dist = p0.distance(p1);
            if dist >= distance {
                if move_t0 {
                    dir0 = dir0 * -0.9;
                }else{
                    dir1 = dir1 * -0.9;
                }
                move_t0 = !move_t0;
            }
            distance = dist;
        }
        if distance < self.tolerance {
            let d0 = curve0.get_vec2_at_u(t0 + 0.001);
            let d1 = curve1.get_vec2_at_u(t1 + 0.001);
            let angle = (d0-p0).angle_between(d1-p1);
            Some(Intersection {
                u: t0,
                angle,
                point: curve0.get_vec2_at_u(t0),
            })
        }else{
            None
        }
    }
}











// let dist0 = curves[*c0].get_vec2_at_u(0.).distance(curves[*c1].get_vec2_at_u(0.));
//                                 let dist1 = curves[*c0].get_vec2_at_u(0.).distance(curves[*c1].get_vec2_at_u(1.));
//                                 let dist2 = curves[*c0].get_vec2_at_u(1.).distance(curves[*c1].get_vec2_at_u(0.));
//                                 let dist3 = curves[*c0].get_vec2_at_u(1.).distance(curves[*c1].get_vec2_at_u(1.));
//                                 if dist0 < tolerance || dist1 < tolerance || dist2 < tolerance || dist3 < tolerance {continue}


// let offsets = vec![
//             vec2(-s, -s),
//             vec2(-s, s),
//             vec2(s, -s),
//             vec2(s, s),
//             vec2(0., -s),
//             vec2(0., s),
//             vec2(-s, 0.),
//             vec2(s, 0.),
//             vec2(0., 0.),
//         ];

// fn get_spatial_key(point: Vec2, cell_size: f32) -> String {
//     (point.x/cell_size).round().to_string() + "," + &(point.y/cell_size).round().to_string()
// }


// //let mut count = 0;
// let mut boundaries: Vec<Vec<Boundary>> = vec![]; //             boundaries.push(vec![]);
// for (i0, curve0) in curves.iter().enumerate() {
//     for (i1, curve1) in curves[i0..].iter().enumerate(){
//         for p0 in curve0.get_controls_as_vec2().windows(2) {// }.enumerate() {
//             for p1 in curve1.get_controls_as_vec2().windows(2) {//}.enumerate() {
//                 let control_intersect = get_line_intersection(p0[0], p0[1], p1[0], p1[1]);
//                 if let Option::Some(ci) = control_intersect {
//                     //shapes.push(Shape::Point([ci.x, ci.y, 0.]));
//                     let vva = search_intersection(&curve0, &curve1, ci);
//                     if let Option::Some(vva) = vva {
//                         let ip0 = curve0.get_vec2_at_u(vva[0]);
//                         let ip1 = curve1.get_vec2_at_u(vva[1]);
//                         shapes.push(Shape::Point([ip0.x, ip0.y, 0.]));
//                         shapes.push(Shape::Point([ip1.x, ip1.y, 0.]));
//                         //console_log!("intersection angle0 {}", vva[2]);
//                         //console_log!("intersection angle1 {}", PI*2.-vva[2]);
//                         let boundary0 = BoundaryV {v:vva[0], angle:vva[2]};
//                         let boundary1 = BoundaryV {v:vva[1], angle:-vva[2]};
//                         boundaries[i0].push(Boundary::V(boundary0));
//                         boundaries[i0+i1].push(Boundary::V(boundary1));
//                     }
//                     //count += 1;
//                 }
//             }
//         }
//     }
// }
// for (i, curve) in curves.iter().enumerate() {
//     let mut nurbs = curve.clone();
//     nurbs.boundaries.extend(boundaries[i].clone());
//     shapes.push(Shape::Curve(nurbs));
// }
// //console_log!("intersections: {}", count);
// shapes