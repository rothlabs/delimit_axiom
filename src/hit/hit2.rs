use crate::{get_line_intersection3, get_line_intersection2, CurveShape, Spatial2};
use glam::*;


//#[derive(Clone)]
pub struct HitTester2 {
    pub curves:    Vec<CurveShape>,
    pub index0:    usize,
    pub index1:    usize,
    pub spatial:   Vec<Spatial2>,
    pub points:    Vec<Vec<Vec2>>,
    pub step:      f32,
    pub tolerance: f32,
}

#[derive(Clone)]
pub struct Hit2 {
    pub u0: f32,
    pub u1: f32,
    pub angle0: f32,
    pub angle1: f32,
    pub center_point: Vec2,
}

impl HitTester2 { 
    pub fn test(&mut self, start_u0: f32, start_u1: f32) -> Option<Hit2> { 
        let curve0 = &self.curves[self.index0];
        let curve1 = &self.curves[self.index1];
        let mut u0 = start_u0;
        let mut u1 = start_u1;
        let mut p0 = curve0.get_point_at_u(u0);
        let mut p1 = curve1.get_point_at_u(u1);
        for _ in 0..10 {
            let tangent_hit = self.get_tangent_hit(u0, u1, p0, p1);
            let (u0_t0, p0_t0) = curve0.get_u_and_point_from_target(u0, tangent_hit - p0);
            let (u1_t0, p1_t0) = curve1.get_u_and_point_from_target(u1, tangent_hit - p1);
            let center = (p0 + p1) / 2.;
            let (u0_t1, p0_t1) = curve0.get_u_and_point_from_target(u0, center - p0);
            let (u1_t1, p1_t1) = curve1.get_u_and_point_from_target(u1, center - p1);
            if p0_t0.distance(p1_t0) < p0_t1.distance(p1_t1) {
                p0 = p0_t0;
                p1 = p1_t0;
                u0 = u0_t0;
                u1 = u1_t0;
            } else {
                p0 = p0_t1;
                p1 = p1_t1;
                u0 = u0_t1;
                u1 = u1_t1;
            }
            if p0.distance(p1) < self.tolerance * 0.5 {
                break;
            }
        }
        None
    }

    pub fn get_tangent_hit(&self, u0: f32, u1: f32, p0: Vec3, p1: Vec3) -> Vec3 {
        let curve0 = &self.curves[self.index0];
        let curve1 = &self.curves[self.index1];
        let tangent0 = curve0.get_tangent_at_u(u0);
        let tangent1 = curve1.get_tangent_at_u(u1);
        get_line_intersection3(p0, tangent0, p1, tangent1) // get_line_intersection(p0, tangent0, p1, tangent1)
    }
}


// if p0.distance(p1) < self.tolerance {
//     let delta = 0.0001;
//     let d0 = u0 + delta;
//     let pd0 = curve0.get_vec2_at_u(d0);
//     let pd1 = curve1.get_vec2_at_u(u1 + delta);
//     if let Some(ip) = get_line_intersection(p0, pd0, p1, pd1) {
//         let ratio = p0.distance(ip) / p0.distance(pd0);
//         let mut u = u0 + (d0-u0)*ratio;
//         let mut point = curve0.get_vec2_at_u(u);
//         let alt_u = u0 + (u0-d0)*ratio;
//         let alt_point = curve0.get_vec2_at_u(alt_u);
//         if alt_point.distance(ip) < point.distance(ip) {
//             u = alt_u;
//             point = alt_point;
//         }
//         let angle = (pd0-p0).angle_between(pd1-p1);
//         Some(Hit2 {
//             u,
//             point,
//             angle,
//         })
//     }else{
//         None
//     }
// }else{
//     None
// }


        // let mut dir0 = curve0.get_param_step(4, self.cell_size/10.);
        // let mut dir1 = curve1.get_param_step(4, self.cell_size/10.);