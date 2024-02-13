use std::{collections::HashMap, f32::{consts::PI, EPSILON, INFINITY, NEG_INFINITY}, fmt::Result};
use crate::{get_curves, log, CurveShape, Model, Shape, SpatialMap};
use serde::{Deserialize, Serialize};
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default)]
struct SampleCell {
    //curves: Vec<CurveSample>,
    curves: Vec<usize>,
    points: Vec<Vec2>,
    params: Vec<f32>,
}

// #[derive(Clone, Default)]
// struct CurveSample {
//     index: usize,
//     points: Vec<Vec2>,
//     params: Vec<f32>,
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Union::default")]
pub struct Union {
    pub parts: Vec<Model>,
}

impl Union { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let cell_size = 5.;
        let cell_map = self.get_cell_map(cell_size);
        let mut intersection_map: SpatialMap<()> = SpatialMap::new(0.025);//HashMap<String, ()> = HashMap::new();
        let mut count = 0.;
        let curves = get_curves(&self.parts);
        shapes.extend(curves.iter().map(|curve| Shape::Curve(curve.clone())));
        //shapes.push(Shape::Curve(curves[*c0].clone()));
        //console_log!("cell_map count: {}", cell_map.map.len());
        //let meta = String::from("");
        let mut key_parts = [0; 2];
        for (cell_key, sample_cell) in &cell_map.map {
            
            let split_key = cell_key.split(",");
            for (i, string_int) in split_key.enumerate() {
                //if i > 1 {continue}
                key_parts[i] = string_int.parse().expect("failed to parse key in union");
            }
            //let num: i32 = split_key.first().unwrap_or(&"0").parse();
            //console_log!("sample_cell curve count: {}", sample_cell.curves.len());
            let tolerance = 0.1;
            for (i0, c0) in sample_cell.curves.iter().enumerate() {
                for x in -1..2 {
                    for y in -1..2 {
                        let key = (key_parts[0]+x).to_string()+","+&(key_parts[1]+y).to_string();// + ",";
                        //console_log!("key {}", cell_key);
                        let sample_cell2 = cell_map.map.get(&key); // get_mut(&vec2(key_parts[0] as f32 + x as f32, key_parts[1] as f32 + y as f32), &meta);
                        if let Some(sample_cell2) = sample_cell2 {
                            for (i1, c1) in sample_cell2.curves.iter().enumerate() {//sample_cell.curves[i0..].iter().enumerate() {
                                if c0 == c1 {continue}
                                let dist0 = curves[*c0].get_vec2_at_u(0.).distance(curves[*c1].get_vec2_at_u(0.));
                                let dist1 = curves[*c0].get_vec2_at_u(0.).distance(curves[*c1].get_vec2_at_u(1.));
                                let dist2 = curves[*c0].get_vec2_at_u(1.).distance(curves[*c1].get_vec2_at_u(0.));
                                let dist3 = curves[*c0].get_vec2_at_u(1.).distance(curves[*c1].get_vec2_at_u(1.));
                                if dist0 < tolerance || dist1 < tolerance || dist2 < tolerance || dist3 < tolerance {continue}
                                        //let intersection_key = c0.to_string() + "," + &c1.to_string() + "," + &cell_key;
                                        //if intersection_map.contains_key(&intersection_key) {continue}
                                        //intersection_map.insert(intersection_key, ());
                                let p0 = sample_cell.points[i0];
                                let p1 = sample_cell2.points[i1];
                                shapes.push(Shape::Point([p0.x, p0.y, 0.]));
                                shapes.push(Shape::Point([p1.x, p1.y, 0.]));
                                // let uua = search_intersection(&curves[*c0], &curves[*c1], sample_cell.params[i0], sample_cell.params[i1], cell_size);
                                // if let Some(uua) = uua {
                                //     if 0. < uua[0] && uua[0] < 1. {
                                //         let ip0 = curves[*c0].get_vec2_at_u(uua[0]);
                                //         let meta = c0.to_string() + "," + &c1.to_string();
                                //         if !intersection_map.contains_key(&ip0, &meta) {
                                //             intersection_map.insert(&ip0, &meta, ());
                                //             shapes.push(Shape::Point([ip0.x, ip0.y, 0.]));
                                //             count += 0.5;
                                //         }
                                //     } 
                                //     if 0. < uua[1] && uua[1] < 1. {
                                //         let ip1 = curves[*c1].get_vec2_at_u(uua[1]);
                                //         let meta = c1.to_string() + "," + &c0.to_string();
                                //         if !intersection_map.contains_key(&ip1, &meta) {
                                //             intersection_map.insert(&ip1, &meta, ());
                                //             shapes.push(Shape::Point([ip1.x, ip1.y, 0.]));
                                //             count += 0.5;
                                //         }
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
        }
        //console_log!("Intersection count! {}", count);
        shapes
    }

    fn get_cell_map(&self, cell_size: f32) -> SpatialMap<SampleCell> {
        let meta = String::from("");
        let s = cell_size;
        //let offset = vec2(s/2., s/2.);
        // let offsets = vec![
        //     vec2(-s, -s),
        //     vec2(-s, s),
        //     vec2(s, -s),
        //     vec2(s, s),
        // ];
        // let offsets = vec![
        //     vec2(-s, -s),
        //     vec2(-s, s),
        //     vec2(s, -s),
        //     vec2(s, s),
        //     vec2(0., -s),
        //     vec2(0., s),
        //     vec2(-s, 0.),
        //     vec2(s, 0.),
        //     vec2(0., 0.),
        // ];
        let mut sample_map: SpatialMap<SampleCell> = SpatialMap::new(cell_size); //let mut sample_map: HashMap<String, SampleCell> = HashMap::new();
        for (i, curve) in get_curves(&self.parts).iter().enumerate() { 
            for u in curve.get_param_samples(1, cell_size*0.75) {
                let point = curve.get_vec2_at_u(u);
                //let key = get_spatial_key(p, cell_size);//(p.x/cell_size).round().to_string() + "," + &(p.y/cell_size).round().to_string();
                if let Some(sample_cell) = sample_map.get_mut(&point, &meta) {
                    //log("add to sample cell!!!");
                    //sample_cell.curves.contains(i)
                    // sample_cell.curves.push(CurveSample{
                    //     index: i,
                    //     points: vec![point],
                    //     params: vec![u],
                    // });
                    sample_cell.curves.push(i);
                    sample_cell.points.push(point);
                    sample_cell.params.push(u);
                }else{
                    sample_map.insert(&point, &meta, &SampleCell {
                        // curves: vec![CurveSample{
                        //     index: i,
                        //     points: vec![point],
                        //     params: vec![u],
                        // }]
                        curves: vec![i],
                        points: vec![point],
                        params: vec![u],
                    });
                }
                // for offset in &offsets {
                //     if let Some(sample_cell) = sample_map.get_mut(&(point + *offset), &meta) {
                //         //log("add to sample cell!!!");
                //         sample_cell.curves.push(i);
                //         sample_cell.points.push(point);
                //         sample_cell.params.push(u);
                //     }else{
                //         sample_map.insert(&point, &meta, &SampleCell {
                //             curves: vec![i],
                //             points: vec![point],
                //             params: vec![u],
                //         });
                //     }
                // }
            }
        }
        sample_map
    }
}

fn search_intersection(curve0: &CurveShape, curve1: &CurveShape, u_start0: f32, u_start1: f32, cell_size: f32) -> Option<[f32; 3]> {
    let tolerance = 0.01; // 0.0025; // approx 0.0001 inch
    let iterations = 10000;
    let mut move_t0 = true; 
    let mut t0 = u_start0;
    let mut t1 = u_start1;
    let mut p0 = curve0.get_vec2_at_u(t0);
    let mut p1 = curve1.get_vec2_at_u(t1);
    let mut distance = p0.distance(p1);
    let mut dir0 = curve0.get_param_step(4, cell_size/4.);
    let mut dir1 = curve1.get_param_step(4, cell_size/4.); 
    for i in 0..iterations {
        if distance < tolerance { 
            break; 
        }
        if i == iterations-1 {
            //log("Hit max iterations in search_intersection!");
        }
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
    if distance < tolerance {
        let d0 = curve0.get_vec2_at_u(t0 + 0.001);
        let d1 = curve1.get_vec2_at_u(t1 + 0.001);
        let angle = (d0-p0).angle_between(d1-p1);
        Some([t0, t1, angle])
    }else{
        None
    }
}

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