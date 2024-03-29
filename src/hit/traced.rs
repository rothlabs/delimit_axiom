use glam::*;
use crate::CurveShape;
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
    index_pairs: Vec<IndexPair>, buf_size: IVec2, traces: Vec<f32>, boxes: Vec<f32>, centers0: Vec<f32>
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
        let mut center = CurveShape::default();
        curve0.negate();
        curve1.negate();
        center.negate();
        let mut points1 = vec![];
        for y in 0..buf_size.y as usize{
            let j = (y * buf_size.x as usize + i) * 4;
            let point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
            if prev_point.distance(point) < 0.05 {break;}
            prev_point = point;
            curve0.controls.push(vec3(traces[j+0], traces[j+1], 0.));
            points1.push(vec3(traces[j+2], traces[j+3], 0.));
            center.controls.push(vec3(centers0[j+0], centers0[j+1], centers0[j+2]));
        }
        points1.reverse();
        curve1.controls.splice(0..0, points1);
        let j = i * 4;
        prev_point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
        let mut points0 = vec![];
        let mut centers1 = vec![];
        for y in 1..buf_size.y as usize{
            let j = (y * buf_size.x as usize + half + i) * 4;
            let point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
            if prev_point.distance(point) < 0.05 {break;}
            prev_point = point;
            points0.push(vec3(traces[j+0], traces[j+1], 0.));
            curve1.controls.push(vec3(traces[j+2], traces[j+3], 0.));
            centers1.push(vec3(centers0[j+0], centers0[j+1], centers0[j+2]));
        }
        points0.reverse();
        centers1.reverse();
        curve0.controls.splice(0..0, points0);
        center.controls.splice(0..0, centers1);
        // for t in 0..center.controls.len()-1 {
        //     if center.controls[t].distance(center.controls[t+1]) < 0.05 {
        //         log("double point on center!!!");
        //     }
        // }
        traced_curves.push(TracedCurve{
            index_pair: index_pairs[i].clone(),
            curve0: curve0.get_valid(),
            curve1: curve1.get_valid(),
            center: center.get_valid(),
        });
    }
    traced_curves
}