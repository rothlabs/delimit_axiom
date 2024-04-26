use glam::*;
use crate::{arrow::*, AT_0_TOL, AT_1_TOL, DELTA_0_TOL, TRACE_STEP};
use crate::{log, CurveShape};
use super::{TestPair, HitPair3};
use std::collections::HashMap;

#[derive(Clone)]
pub struct TracedCurve {
    pub index_pair: TestPair,
    pub curve0: CurveShape,
    pub curve1: CurveShape,
    pub center: CurveShape,
}

//impl TracedCurve {
pub fn get_traced_curves(
    hit_pairs: Vec<TestPair>, buf_size: IVec2, traces: Vec<f32>, boxes: Vec<f32>, 
    centers0: Vec<f32>, uv_dirs: Vec<f32>, dirs: Vec<f32>, 
) -> Vec<HitPair3> {
    let mut traced_curves = vec![];
    let mut box_map: HashMap<String, Vec<(Vec2, Vec2)>> = HashMap::new();
    let half = buf_size.x as usize / 2;
    //let half_dial = (dual_buf_size.x * dual_buf_size.y) as usize * 2;
    // let index_len = index_pairs.len();
    //console_log!("buf_size.x, index_pairs.len(), {}, {}", buf_size.x, index_pairs.len());
    for i in 0..hit_pairs.len() {
        let min = vec2(boxes[i*4], boxes[i*4+1]).min(vec2(boxes[(half+i)*4], boxes[(half+i)*4+1]));
        let max = vec2(boxes[i*4+2], boxes[i*4+3]).max(vec2(boxes[(half+i)*4+2], boxes[(half+i)*4+3]));
        //let key = index_pairs[i].g0.to_string() + ":" + &index_pairs[i].i0.to_string();
        let key = hit_pairs[i].i0.to_string();
        let mut duplicate = false;
        if box_map.contains_key(&key) {
            for (min1, max1) in box_map.get(&key).unwrap() {
                if min.distance(*min1) < 0.05 && max.distance(*max1) < 0.05 {
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
        
        // if index_pairs[i].g0 > index_pairs[i].g1 {
        //     log("What?!!!!!!!!!!!");
        // }
        // //console_log!("group pair: {}, {}", index_pairs[i].g0, index_pairs[i].g1);
        // console_log!("signs: {}, {}", facet_groups[index_pairs[i].g0][index_pairs[i].i0].nurbs.sign, 
        //     facet_groups[index_pairs[i].g1][index_pairs[i].i1].nurbs.sign);
        let j = i * 4;
        let starts = (
            Arrow{
                point: vec3(traces[j], traces[j+1], 0.),
                delta: vec3(uv_dirs[j], uv_dirs[j+1], 0.),
            },
            Arrow{
                point: vec3(traces[j+2], traces[j+3], 0.),
                delta: vec3(uv_dirs[j+2], uv_dirs[j+3], 0.),
            },
            Arrow{ 
                point: vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                delta: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
            }
        );

        let mut can_loop = false;
        let mut add_last_arrow_and_add_reverse = true;
        let mut rays0a = vec![];
        let mut rays1a = vec![];
        let mut rays2a = vec![];
        for y in 0..buf_size.y as usize{
            let j = (y * buf_size.x as usize + i) * 4; 
            let delta0 = vec3(uv_dirs[j+0], uv_dirs[j+1], 0.).normalize();
            let delta1 = vec3(uv_dirs[j+2], uv_dirs[j+3], 0.).normalize();
            if       (delta0.x > DELTA_0_TOL && traces[j+0] > AT_1_TOL) || (delta0.x < -DELTA_0_TOL && traces[j+0] < AT_0_TOL) { 
                break;
            }else if (delta0.y > DELTA_0_TOL && traces[j+1] > AT_1_TOL) || (delta0.y < -DELTA_0_TOL && traces[j+1] < AT_0_TOL) { 
                break;
            }else if (delta1.x > DELTA_0_TOL && traces[j+2] > AT_1_TOL) || (delta1.x < -DELTA_0_TOL && traces[j+2] < AT_0_TOL) { 
                break;
            }else if (delta1.y > DELTA_0_TOL && traces[j+3] > AT_1_TOL) || (delta1.y < -DELTA_0_TOL && traces[j+3] < AT_0_TOL) { 
                break;
            }
            let point = vec3(centers0[j+0], centers0[j+1], centers0[j+2]);
            if starts.2.point.distance(point) < TRACE_STEP {
                if can_loop {
                    if starts.2.delta.normalize().dot((point - starts.2.point).normalize()) > 0.5 {
                        add_last_arrow_and_add_reverse = false;
                        rays0a.push(starts.0.clone());
                        rays1a.push(starts.1.clone());
                        rays2a.push(starts.2);
                        break;
                    }
                }
            }else{
                can_loop = true;
            }
            //curve0.controls.push(vec3(traces[j+0], traces[j+1], 0.));
            rays0a.push(Arrow{ 
                point: vec3(traces[j+0], traces[j+1], 0.),
                delta: delta0 // vec3(uv_dirs[j+0], uv_dirs[j+1], 0.),
            });
            //rays1a.push(vec3(traces[j+2], traces[j+3], 0.));
            rays1a.push(Arrow{ 
                point: vec3(traces[j+2], traces[j+3], 0.),
                delta: delta1 // vec3(uv_dirs[j+2], uv_dirs[j+3], 0.), // was negated
            });
            rays2a.push(Arrow{ 
                point, // : vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                delta: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
            });
        }
        if add_last_arrow_and_add_reverse {
            let j = ((buf_size.y-1) as usize * buf_size.x as usize + i) * 4; 
            rays0a.push(Arrow{ 
                point: vec3(traces[j+0], traces[j+1], 0.),
                delta: vec3(uv_dirs[j+0], uv_dirs[j+1], 0.),
            });
            rays1a.push(Arrow{ 
                point: vec3(traces[j+2], traces[j+3], 0.),
                delta: vec3(uv_dirs[j+2], uv_dirs[j+3], 0.), // was negated
            });
            rays2a.push(Arrow{ 
                point: vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                delta: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
            });
            
            rays1a.reverse();


            // let mut points0 = vec![];
            let mut add_reverse_trace = true;
            if rays0a.len() > 1 {
                let j = i * 4;
                let delta0 = starts.0.delta.normalize();//vec3(uv_dirs[j+0], uv_dirs[j+1], 0.).normalize();
                let delta1 = starts.1.delta.normalize();////vec3(uv_dirs[j+2], uv_dirs[j+3], 0.).normalize();
                if       delta0.x.abs() > DELTA_0_TOL && (traces[j+0] < AT_0_TOL || traces[j+0] > AT_1_TOL) { 
                    add_reverse_trace = false;
                }else if delta0.y.abs() > DELTA_0_TOL && (traces[j+1] < AT_0_TOL || traces[j+1] > AT_1_TOL) { 
                    add_reverse_trace = false;
                }else if delta1.x.abs() > DELTA_0_TOL && (traces[j+2] < AT_0_TOL || traces[j+2] > AT_1_TOL) { 
                    add_reverse_trace = false;
                }else if delta1.y.abs() > DELTA_0_TOL && (traces[j+3] < AT_0_TOL || traces[j+3] > AT_1_TOL) { 
                    add_reverse_trace = false;
                }
            }
            if add_reverse_trace {
                let mut rays0b = vec![];
                let mut rays1b = vec![];
                let mut rays2b = vec![];
                for y in 1..buf_size.y as usize {
                    let j = (y * buf_size.x as usize + half + i) * 4;
                    let delta0 = -vec3(uv_dirs[j+0], uv_dirs[j+1], 0.).normalize();
                    let delta1 = -vec3(uv_dirs[j+2], uv_dirs[j+3], 0.).normalize();
                    if       (delta0.x > DELTA_0_TOL && traces[j+0] > AT_1_TOL) || (delta0.x < -DELTA_0_TOL && traces[j+0] < AT_0_TOL) { 
                        break;
                    }else if (delta0.y > DELTA_0_TOL && traces[j+1] > AT_1_TOL) || (delta0.y < -DELTA_0_TOL && traces[j+1] < AT_0_TOL) { 
                        break;
                    }else if (delta1.x > DELTA_0_TOL && traces[j+2] > AT_1_TOL) || (delta1.x < -DELTA_0_TOL && traces[j+2] < AT_0_TOL) { 
                        break;
                    }else if (delta1.y > DELTA_0_TOL && traces[j+3] > AT_1_TOL) || (delta1.y < -DELTA_0_TOL && traces[j+3] < AT_0_TOL) { 
                        break;
                    }
                    //points0.push(vec3(traces[j+0], traces[j+1], 0.));
                    rays0b.push(Arrow{ 
                        point: vec3(traces[j+0],  traces[j+1],  0.),
                        delta: -delta0 // vec3(uv_dirs[j+0], uv_dirs[j+1], 0.), // was negated
                    });
                    //curve1.controls.push(vec3(traces[j+2], traces[j+3], 0.));
                    rays1b.push(Arrow{ 
                        point: vec3(traces[j+2],  traces[j+3],  0.),
                        delta: -delta1 // vec3(uv_dirs[j+2], uv_dirs[j+3], 0.),
                    });
                    //centers1.push(vec3(centers0[j+0], centers0[j+1], centers0[j+2]));
                    rays2b.push(Arrow{ 
                        point: vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                        delta: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
                    });
                }
            //if add_last_point {
                let j = ((buf_size.y-1) as usize * buf_size.x as usize + half + i) * 4; 
                rays0b.push(Arrow{ 
                    point: vec3(traces[j+0],  traces[j+1],  0.),
                    delta: vec3(uv_dirs[j+0], uv_dirs[j+1], 0.), // was negated
                });
                rays1b.push(Arrow{ 
                    point: vec3(traces[j+2],  traces[j+3],  0.),
                    delta: vec3(uv_dirs[j+2], uv_dirs[j+3], 0.),
                });
                rays2b.push(Arrow{ 
                    point: vec3(centers0[j+0], centers0[j+1], centers0[j+2]),
                    delta: vec3(dirs[j+0], dirs[j+1], dirs[j+2]),
                });
                rays0b.reverse();
                rays0a.splice(0..0, rays0b);
                rays1a.extend(rays1b);
                rays2b.reverse();
                rays2a.splice(0..0, rays2b);
            }
        }

        // let IndexPair{g0, i0, g1, i1} = index_pairs[i];
        // if facet_groups[g0][i0].nurbs.sign < 0. {
        //     rays0a.reverse();
        //     rays1a.reverse();
        // }
        if hit_pairs[i].reverse {
            rays0a.reverse();
            rays1a.reverse();
        }
        

        // let mut curve0 = CurveShape::default();
        // let mut curve1 = CurveShape::default();
        // let mut curve2 = CurveShape::default();
        // curve0.controls.extend(rays0a.iter().map(|x| x.point));
        // curve1.controls.extend(rays1a.iter().map(|x| x.point));
        // curve2.controls.extend(rays2a.iter().map(|x| x.point));
        let mut curve0 = rays0a.to_curve();
        let mut curve1 = rays1a.to_curve();
        let mut curve2 = rays2a.to_curve();
        curve0.negate();
        curve1.negate();
        curve0 = curve0.get_valid();
        curve1 = curve1.get_valid();
        curve2 = curve2.get_valid();
        traced_curves.push(HitPair3{
            pair: hit_pairs[i].clone(),
            curve0,
            curve1,
            curve2, 
        });
    }
    traced_curves
}


// for t in 0..rays0a.len()-1 {
//     if rays0a[t].point.distance(rays0a[t+1].point) < 0.005 {
//         console_log!("double point on rays0a!!! {}", rays0a[t].point.distance(rays0a[t+1].point));
//     }
// }
// for t in 0..rays1a.len()-1 {
//     if rays1a[t].point.distance(rays1a[t+1].point) < 0.005 {
//         console_log!("double point on rays1a!!! {}", rays1a[t].point.distance(rays1a[t+1].point));
//     }
// }
// for t in 0..rays2a.len()-1 {
//     if rays2a[t].point.distance(rays2a[t+1].point) < 0.05 {
//         console_log!("double point on rays2a!!! {}", rays2a[t].point.distance(rays2a[t+1].point));
//     }
// }

// if rays0a.len() < 3 {
//     console_log!("rays0: {}", rays0a.len());
//     continue;
// }
// if rays1a.len() < 3 {
//     console_log!("rays1: {}", rays1a.len());
//     continue;
// }
// if rays2a.len() < 3 {
//     console_log!("rays2: {}", rays2a.len());
//     continue;
// }