use super::union2::UnionBasis2;
use glam::*;


#[derive(Clone)]
pub struct Hit2 {
    pub u: f32,
    pub angle: f32,
    pub point: Vec2,
}

impl UnionBasis2 { 
    pub fn get_hit(&self, c0: &usize, c1: &usize, u_start0: f32, u_start1: f32) -> Option<Hit2> {
        let curve0 = &self.curves[*c0];
        let curve1 = &self.curves[*c1];
        let mut move_u0 = true; 
        let mut u0 = u_start0;
        let mut u1 = u_start1;
        let mut p0 = curve0.get_vec2_at_u(u0);
        let mut p1 = curve1.get_vec2_at_u(u1);
        let mut dir0 = self.curve_params.get(c0).unwrap().step / 10.;
        let mut dir1 = self.curve_params.get(c1).unwrap().step / 10.;
        let mut distance = p0.distance(p1);
        for _ in 0..self.max_walk_iterations {
            if distance < self.tolerance { 
                break; 
            }
            // if i == self.max_walk_iterations-1 {
            //     log("Hit max iterations in get_intersection!");
            // }
            if move_u0 {
                u0 = (u0 + dir0).clamp(0., 1.);
                p0 = curve0.get_vec2_at_u(u0);
            }else{
                u1 = (u1 + dir1).clamp(0., 1.);
                p1 = curve1.get_vec2_at_u(u1);
            }
            let dist = p0.distance(p1);
            if dist >= distance {
                if move_u0 {
                    dir0 = dir0 * -0.99;
                }else{
                    dir1 = dir1 * -0.99;
                }
                move_u0 = !move_u0;
            }
            distance = dist;
        }
        if distance < self.tolerance {
            let delta = 0.0001;
            let d0 = u0 + delta;
            let pd0 = curve0.get_vec2_at_u(d0);
            let pd1 = curve1.get_vec2_at_u(u1 + delta);
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
                Some(Hit2 {
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


        // let mut dir0 = curve0.get_param_step(4, self.cell_size/10.);
        // let mut dir1 = curve1.get_param_step(4, self.cell_size/10.);