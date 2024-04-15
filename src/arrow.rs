use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8};
use glam::*;
use crate::{log, CurveShape};

const TWO_ARROWS: &str = "There should be two arrows or more.";

#[derive(Default, Clone)]
pub struct Arrow {
    pub point: Vec3,
    pub delta: Vec3,
}

impl Arrow {
    pub fn new(origin: Vec3, vector: Vec3) -> Self {
        Self {point: origin, delta: vector}
    }
    pub fn middle(&self, arrow: &Arrow) -> Vec3 {
        let v0 = self.delta.normalize();
        let v1 = arrow.delta.normalize();
        //if v0.dot(v1) > 0.95 { 
        if v0.cross(v1).length() < 0.001 { // parallel case
            return (self.point + arrow.point) / 2.;
        }
        let a = v0.dot(v0);
        let b = v0.dot(v1);
        let c = v1.dot(v1);
        let delta = self.point - arrow.point;
        let d = v0.dot(delta);
        let e = v1.dot(delta);
        let denom = a * c - b * b;
        let u0 = (b * e - c * d) / denom;
        let u1 = (a * e - b * d) / denom;
        let p0 = self.point + v0 * u0;
        let p1 = arrow.point  + v1  * u1;

        let point = (p0 + p1) / 2.;
        if point.is_nan() {
            log("arrow.middle -> nan!");
        }
        point
        //(p0 + p1) / 2.
    }
}

#[derive(Default, Clone)]
pub struct SpaceArrow {
    pub point: Vec3,
    pub world_delta: Vec3,
    pub local_delta: Vec3,
}

impl SpaceArrow {
    pub fn new(point: Vec3, world_delta: Vec3, local_delta: Vec3) -> Self {
        Self {point, world_delta, local_delta}
    }
    pub fn middle(&self, arrow: &SpaceArrow) -> Vec3 {
        let v0 = self.local_delta.normalize();
        let v1 = arrow.local_delta.normalize();
        //if v0.dot(v1) > 0.95 { 
        if v0.cross(v1).length() < 0.001 { // parallel case
            return (self.point + arrow.point) / 2.;
        }
        let a = v0.dot(v0);
        let b = v0.dot(v1);
        let c = v1.dot(v1);
        let delta = self.point - arrow.point;
        let d = v0.dot(delta);
        let e = v1.dot(delta);
        let denom = a * c - b * b;
        let u0 = (b * e - c * d) / denom;
        let u1 = (a * e - b * d) / denom;
        let p0 = self.point + v0 * u0;
        let p1 = arrow.point  + v1  * u1;

        let point = (p0 + p1) / 2.;
        if point.is_nan() {
            log("arrow.middle -> nan!");
        }
        point
        //(p0 + p1) / 2.
    }
}


pub trait ToCurve {
    fn to_curve(self) -> CurveShape;
}

impl ToCurve for Vec<SpaceArrow> {
    fn to_curve(self) -> CurveShape {
        ArrowsToCurve {
            curve: CurveShape::from_order(3),
            arrow: self.first().expect(TWO_ARROWS).clone(),
            knot:  0.,
        }.make(self)
    }
}

pub struct ArrowsToCurve {
    curve: CurveShape,
    knot:  f32,
    arrow: SpaceArrow,
}

impl ArrowsToCurve {
    fn make(&mut self, arrows: Vec<SpaceArrow>) -> CurveShape {
        self.curve.controls.push(self.arrow.point);
        self.curve.nurbs.weights.push(1.);
        let delta = arrows.get(1).expect(TWO_ARROWS).local_delta;
        let mut base_angle = self.arrow.local_delta.angle_between(delta);
        for (i, arrow) in arrows.windows(2).enumerate() {
            if i > 0 {
                // console_log!("arrow[1].point {}", arrow[1].point);
                // console_log!("arrow[1].delta {}", arrow[1].delta);
                let angle = self.arrow.local_delta.angle_between(arrow[1].local_delta);
                //if angle > FRAC_PI_8 || (angle > 0.01 && angle < base_angle) { // 1/8th turn or inflection 
                if angle > FRAC_PI_8 || angle < base_angle - 0.001 {
                    self.add_arc(&arrow[0]);
                    base_angle = 0.;
                }else{
                    base_angle = angle;
                }
                if i+3 > arrows.len() {
                    self.add_arc(&arrow[1]);
                }
            }
        }
        self.curve.nurbs.knots.push(self.knot);
        self.curve.nurbs.normalize_knots();
        self.curve.clone()
    }
    fn add_arc(&mut self, arrow: &SpaceArrow) { 
        let middle = self.arrow.middle(arrow);
        self.curve.controls.push(middle); 
        self.curve.controls.push(arrow.point);
        // if (self.arrow.point - middle).length() == 0. {
        //     log("self.arrow and middle the same");
        //     console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
        // }
        // if (arrow.point - middle).length() == 0. {
        //     log("arrow and middle the same");
        //     console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
        // }
        self.knot += (arrow.point - self.arrow.point).length();
        //self.knot += 1.;
        self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
        let angle = self.arrow.world_delta.angle_between(arrow.world_delta);
        self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]);  // (angle / 2.).cos()
        self.arrow = arrow.clone();
    }
}






// impl ArrowsToCurve {
//     fn make(&mut self, arrows: Vec<SpaceArrow>) -> CurveShape {
//         self.curve.controls.push(self.arrow.point);
//         self.curve.nurbs.weights.push(1.);
//         let vector = arrows.get(1).expect(TWO_ARROWS).world_delta;
//         let mut base_angle = self.arrow.world_delta.angle_between(vector);
//         for (i, arrow) in arrows.windows(2).enumerate() {
//             if i > 0 {
//                 console_log!("arrow.point {}", arrow[1].point);
//                 let angle = self.arrow.world_delta.angle_between(arrow[1].world_delta);
//                 if angle > FRAC_PI_4 || (angle > 0.1 && angle < base_angle) { // 1/8th turn or inflection 
//                     self.add_arc(&arrows[0]);
//                     base_angle = 0.;
//                 }else{
//                     base_angle = angle;
//                 }
//                 if i+3 > arrows.len() {
//                     self.add_arc(&arrows[1]);
//                 }
//             }
//         }
//         self.curve.nurbs.knots.push(self.knot);
//         self.curve.nurbs.normalize_knots();
//         self.curve.clone()
//     }
//     fn add_arc(&mut self, arrow: &SpaceArrow) { 
//         let middle = self.arrow.middle(arrow);
//         self.curve.controls.push(middle); 
//         self.curve.controls.push(arrow.point);
//         if (self.arrow.point - middle).length() == 0. {
//             log("self.arrow and middle the same");
//             console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
//         }
//         // if (arrow.point - middle).length() == 0. {
//         //     log("arrow and middle the same");
//         //     console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
//         // }
//         self.knot += (arrow.point - self.arrow.point).length();
//         //self.knot += 1.;
//         self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
//         let angle = self.arrow.world_delta.angle_between(arrow.world_delta);
//         self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]);  
//         self.arrow = arrow.clone();
//     }
// }








// pub fn new(rays: Vec<Ray>) -> CurveShape {
    //     Self {
    //         curve: CurveShape::from_order(3),
    //         ray: rays.first().expect(TWO_RAYS).clone(),
    //         knot:  0.,
    //     }.make(rays)
    // }


// if self.ray.origin.distance(ray.origin) < 0.005 {
        //     let len = &self.curve.controls.len() - 1;
        //     if len > 2 {
        //         log("short move in RaysToCurve");
        //         self.curve.controls[len - 1] = ray.origin;
        //         self.ray = ray.clone();
        //     } else {
        //         //log("tried to do short move in RaysToCurve but not enough control points");
        //         if remaining > 0 {
        //             log("skip point");
        //         }else{
        //             self.curve.controls.push(self.ray.middle(ray)); 
        //             self.curve.controls.push(ray.origin);
        //             self.knot += (ray.origin - self.ray.origin).length();
        //             self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
        //             let angle = self.ray.vector.angle_between(ray.vector);
        //             self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]); 
        //             self.ray = ray.clone();
        //         }

        //         // //self.curve.controls.push((self.ray.origin + ray.origin) / 2.); 
        //         // //self.curve.controls.push(ray.origin);
        //         // self.curve.controls = vec![self.ray.origin, ray.origin];
        //         // self.knot = 1.;
        //         // self.curve.nurbs.knots = vec![0., 0., self.knot, self.knot];
        //         // self.curve.nurbs.weights= vec![1., 1.]; 
        //         // self.curve.nurbs.order = 2;
        //     }
        // }else{