use std::{collections::HashMap, f32::{consts::PI, EPSILON, INFINITY, NEG_INFINITY}};
use crate::{get_curves, log, Boundary, Model, Nurbs, Shape, BoundaryV};
use serde::{Deserialize, Serialize};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default)]
struct SampleCell {
    curves: Vec<usize>,
    points: Vec<Vec2>,
    params: Vec<f32>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Union::default")]
pub struct Union {
    pub parts: Vec<Model>,
}

impl Union { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let sample_map = self.get_sample_map();
        //let mut sample_cells = vec![];
        //shapes.extend(get_curves(&self.parts).iter().map(|nurbs| Shape::Curve(nurbs.clone())));
        let curves = get_curves(&self.parts);
        //let mut count = 0;
        let mut boundaries: Vec<Vec<Boundary>> = vec![]; //             boundaries.push(vec![]);
        for (i0, curve0) in curves.iter().enumerate() {
            for (i1, curve1) in curves[i0..].iter().enumerate(){
                for p0 in curve0.get_controls_as_vec2().windows(2) {// }.enumerate() {
                    for p1 in curve1.get_controls_as_vec2().windows(2) {//}.enumerate() {
                        let control_intersect = get_line_intersection(p0[0], p0[1], p1[0], p1[1]);
                        if let Option::Some(ci) = control_intersect {
                            //shapes.push(Shape::Point([ci.x, ci.y, 0.]));
                            let vva = search_intersection(&curve0, &curve1, ci);
                            if let Option::Some(vva) = vva {
                                let ip0 = curve0.get_vec2_at_u(vva[0]);
                                let ip1 = curve1.get_vec2_at_u(vva[1]);
                                shapes.push(Shape::Point([ip0.x, ip0.y, 0.]));
                                shapes.push(Shape::Point([ip1.x, ip1.y, 0.]));
                                //console_log!("intersection angle0 {}", vva[2]);
                                //console_log!("intersection angle1 {}", PI*2.-vva[2]);
                                let boundary0 = BoundaryV {v:vva[0], angle:vva[2]};
                                let boundary1 = BoundaryV {v:vva[1], angle:-vva[2]};
                                boundaries[i0].push(Boundary::V(boundary0));
                                boundaries[i0+i1].push(Boundary::V(boundary1));
                            }
                            //count += 1;
                        }
                    }
                }
            }
        }
        for (i, curve) in curves.iter().enumerate() {
            let mut nurbs = curve.clone();
            nurbs.boundaries.extend(boundaries[i].clone());
            shapes.push(Shape::Curve(nurbs));
        }
        //console_log!("intersections: {}", count);
        shapes
    }

    fn get_sample_map(&self) -> HashMap<String, SampleCell> {
        let mut sample_map: HashMap<String, SampleCell> = HashMap::new();
        let sample_count = 100;
        let cell_size = 10.;
        for (i, curve) in get_curves(&self.parts).iter().enumerate() { 
            for step in 0..sample_count {
                let v = step as f32 / (sample_count - 1) as f32;
                let p = curve.get_vec2_at_u(v);
                let key = (p.x/cell_size).round().to_string() + "," + &(p.y/cell_size).round().to_string();
                if let Some(sample_cell) = sample_map.get_mut(&key) {
                    sample_cell.curves.push(i);
                    sample_cell.points.push(p);
                    sample_cell.params.push(v);
                }else{
                    sample_map.insert(key, SampleCell {
                        curves: vec![i],
                        points: vec![p],
                        params: vec![v],
                    });
                }
            }
        }
        sample_map
    }
}

fn search_intersection(nurbs0: &Nurbs, nurbs1: &Nurbs, proxy: Vec2) -> Option<[f32; 3]> {
    let tolerance = 0.1;
    let iterations = 10000;
    let mut move_t0 = true; 
    let mut t0 = 0.;//nurbs0.knots[nurbs0.order + start_i0];
    let mut t1 = 0.;//nurbs1.knots[nurbs1.order + start_i1];
    let mut p0 = vec2(0., 0.);//nurbs0.get_vec2_at_v(t0);
    let mut p1 = vec2(0., 0.);//nurbs1.get_vec2_at_v(t1);
    let mut dist0 = INFINITY;
    let mut dist1 = INFINITY;
    let proxy_search_count = 400;
    for t in 0..proxy_search_count {
        let tt = t as f32 / (proxy_search_count - 1) as f32;
        p0 = nurbs0.get_vec2_at_v(tt);
        let dist = p0.distance(proxy);
        if dist0 > dist {dist0 = dist; t0 = tt;}
        p1 = nurbs1.get_vec2_at_v(tt);
        let dist = p1.distance(proxy);
        if dist1 > dist {dist1 = dist; t1 = tt;}
    }
    let mut distance = p0.distance(p1);
    let mut dir0 = 1. / proxy_search_count as f32; // (1. - t0) / 2.; //iterations as f32 / 2.;
    let mut dir1 = 1. / proxy_search_count as f32; // (1. - t1) / 2.; //iterations as f32 / 2.;
    //if t0 > 0.5 { dir0 = - t0 / 2. }
    //if t1 > 0.5 { dir1 = - t1 / 2. }
    for i in 0..iterations {
        if distance < tolerance { 
            break; 
        }
        if i == iterations-1 {
            log("Hit max iterations in search_intersection!");
        }
        if move_t0 {
            t0 = (t0 + dir0).clamp(0., 1.);
            p0 = nurbs0.get_vec2_at_v(t0);
        }else{
            t1 = (t1 + dir1).clamp(0., 1.);
            p1 = nurbs1.get_vec2_at_v(t1);
        }
        let dist = p0.distance(p1);
        if dist >= distance {
            if move_t0 {
                dir0 = - dir0 * 0.9;
            }else{
                dir1 = - dir1 * 0.9;
            }
            move_t0 = !move_t0;
        }
        distance = dist;
    }
    if distance < tolerance {
        let d0 = nurbs0.get_vec2_at_v(t0 + 0.001);
        let d1 = nurbs1.get_vec2_at_v(t1 + 0.001);
        let angle = (d0-p0).angle_between(d1-p1);
        Some([t0, t1, angle])
    }else{
        None
    }
}

