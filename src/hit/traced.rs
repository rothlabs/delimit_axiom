use glam::*;
use crate::{log, rays_to_curve::RaysToCurve, CurveShape, Ray};
use super::IndexPair;
use std::collections::HashMap;

pub struct TracedCurve {
    pub index_pair: IndexPair,
    pub curve0: CurveShape,
    pub curve1: CurveShape,
    pub center: CurveShape,
}

//impl TracedCurve {
pub fn get_traced_curves(
    index_pairs: Vec<IndexPair>, buf_size: IVec2, traces: Vec<f32>, boxes: Vec<f32>, 
    centers0: Vec<f32>, uv_dirs: Vec<f32>, dirs: Vec<f32>, 
) -> Vec<TracedCurve> {
    let mut traced_curves = vec![];
    let mut box_map: HashMap<String, Vec<(Vec2, Vec2)>> = HashMap::new();
    let half = buf_size.x as usize / 2;
    for i in 0..index_pairs.len() {
        let min = vec2(boxes[i*4], boxes[i*4+1]).min(vec2(boxes[(half+i)*4], boxes[(half+i)*4+1]));
        let max = vec2(boxes[i*4+2], boxes[i*4+3]).max(vec2(boxes[(half+i)*4+2], boxes[(half+i)*4+3]));
        let key = index_pairs[i].g0.to_string() + ":" + &index_pairs[i].i0.to_string();
        let mut duplicate = false;
        if box_map.contains_key(&key) {
            for (min1, max1) in box_map.get(&key).unwrap() {
                if min.distance(*min1) < 0.01 && max.distance(*max1) < 0.01 {
                    duplicate = true;
                }
            }
        }else{
            box_map.insert(key.clone(), vec![]);
        }
        if duplicate {continue}
        if let Some(vec) = box_map.get_mut(&key) {
            vec.push((min, max));
        }
        let mut prev_point = vec3(100000., 100000., 100000.);
        let mut curve0 = CurveShape::default();
        let mut curve1 = CurveShape::default();
        let mut rays0a = vec![];
        let mut rays1a = vec![];
        let mut rays2a = vec![];
        for y in 0..buf_size.y as usize{
            let j = (y * buf_size.x as usize + i) * 4;
            let point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
            if prev_point.distance(point) < 0.05 {break;}
            prev_point = point;
            //curve0.controls.push(vec3(traces[j+0], traces[j+1], 0.));
            rays0a.push(Ray{ 
                origin: vec3(traces[j+0], traces[j+1], 0.),
                vector: vec3(uv_dirs[j+0], uv_dirs[j+1], 0.),
            });
            //rays1a.push(vec3(traces[j+2], traces[j+3], 0.));
            rays1a.push(Ray{ 
                origin: vec3(traces[j+2], traces[j+3], 0.),
                vector: -vec3(uv_dirs[j+2], uv_dirs[j+3], 0.),
            });
            //center.controls.push(vec3(centers0[j+0], centers0[j+1], centers0[j+2]));
            rays2a.push(Ray{ 
                origin: vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                vector: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
            });
        }
        rays1a.reverse();

        let j = i * 4;
        prev_point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
        // let mut points0 = vec![];
        // let mut centers1 = vec![];
        let mut rays0b = vec![];
        let mut rays1b = vec![];
        let mut rays2b = vec![];
        for y in 1..buf_size.y as usize{
            let j = (y * buf_size.x as usize + half + i) * 4;
            let point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
            if prev_point.distance(point) < 0.05 {break;}
            prev_point = point;
            //points0.push(vec3(traces[j+0], traces[j+1], 0.));
            rays0b.push(Ray{ 
                origin:  vec3(traces[j+0],  traces[j+1],  0.),
                vector: -vec3(uv_dirs[j+0], uv_dirs[j+1], 0.),
            });
            //curve1.controls.push(vec3(traces[j+2], traces[j+3], 0.));
            rays1b.push(Ray{ 
                origin: vec3(traces[j+2],  traces[j+3],  0.),
                vector: vec3(uv_dirs[j+2], uv_dirs[j+3], 0.),
            });
            //centers1.push(vec3(centers0[j+0], centers0[j+1], centers0[j+2]));
            rays2b.push(Ray{ 
                origin:  vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                vector: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
            });
        }
        rays0b.reverse();
        rays0a.splice(0..0, rays0b);
        rays1a.extend(rays1b);
        rays2b.reverse();
        rays2a.splice(0..0, rays2b);
        //curve0.controls.splice(0..0, points0);
        
        // for t in 0..rays0a.len()-1 {
        //     if rays0a[t].origin.distance(rays0a[t+1].origin) < 0.0005 {
        //         log("double point on rays0a!!!");
        //     }
        // }
        // for t in 0..rays1a.len()-1 {
        //     if rays1a[t].origin.distance(rays1a[t+1].origin) < 0.0005 {
        //         log("double point on rays1a!!!");
        //     }
        // }
        // for t in 0..rays2a.len()-1 {
        //     if rays2a[t].origin.distance(rays2a[t+1].origin) < 0.05 {
        //         log("double point on rays2a!!!");
        //     }
        // }
        if rays0a.len() < 3 {
            console_log!("rays0: {}", rays0a.len());
            continue;
        }
        if rays1a.len() < 3 {
            console_log!("rays1: {}", rays1a.len());
            continue;
        }
        if rays2a.len() < 3 {
            console_log!("rays2: {}", rays2a.len());
            continue;
        }
        //if duplicate {continue}
        for i in 0..rays0a.len()-1 {
            if rays0a[i+1].vector.is_nan() {
                rays0a[i+1].vector = rays0a[i].vector;
            }else if rays0a[i].vector.is_nan() {
                rays0a[i].vector = rays0a[i+1].vector;
            }
        }
        for i in 0..rays1a.len()-1 {
            if rays1a[i+1].vector.is_nan() {
                rays1a[i+1].vector = rays1a[i].vector;
            }else if rays1a[i].vector.is_nan() {
                rays1a[i].vector = rays1a[i+1].vector;
            }
        }
        for i in 0..rays0a.len() {
            if rays0a[i].vector.is_nan() {
                log("rays0a nan!!!");
            }
        }
        for i in 0..rays1a.len() {
            if rays1a[i].vector.is_nan() {
                log("rays1a nan!!!");
            }
        }
        // console_log!("dirs0 {:?}", rays0a.iter().map(|x| x.vector).collect::<Vec<Vec3>>());
        // console_log!("dirs1 {:?}", rays1a.iter().map(|x| x.vector).collect::<Vec<Vec3>>());
        // curve0.controls.extend(rays0a.iter().map(|x| x.origin));
        // curve1.controls.extend(rays1a.iter().map(|x| x.origin));
        curve0 = RaysToCurve::new(rays0a);
        curve1 = RaysToCurve::new(rays1a);
        curve0.negate();
        curve1.negate();
        curve0 = curve0.get_valid();
        curve1 = curve1.get_valid();
        // console_log!("knots0 {:?}", curve0.nurbs.knots);
        // console_log!("knots1 {:?}", curve1.nurbs.knots);
        traced_curves.push(TracedCurve{
            index_pair: index_pairs[i].clone(),
            // curve0: curve0.get_valid(), 
            // curve1: curve1.get_valid(), 
            // curve0: RaysToCurve::new(rays0a),
            // curve1: RaysToCurve::new(rays1a),
            curve0,
            curve1,
            center: RaysToCurve::new(rays2a),//center.get_valid(),
        });
    }
    traced_curves
}