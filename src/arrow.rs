use std::f32::consts::FRAC_PI_8;
use glam::*;
use crate::{log, DOT_1_TOL, Shape};

const TWO_ARROWS: &str = "There should be two arrows or more.";

#[derive(Default, Clone)]
pub struct Arrow {
    pub point: Vec3,
    pub delta: Vec3,
}

impl Arrow {
    pub fn new(point: Vec3, delta: Vec3) -> Self {
        Self {point, delta}
    }
    pub fn middle(&self, arrow: &Arrow) -> Vec3 {
        let delta0 = self.delta.normalize();
        let delta1 = arrow.delta.normalize();
        let dotx = delta0.dot(delta1);
        if dotx.abs() > DOT_1_TOL { 
            return (self.point + arrow.point) / 2.;
        }
        // let delta = self.point - arrow.point;
        // let dot0 = delta.dot(delta0);
        // let dot1 = delta.dot(delta1);
        let denom = 1. - dotx * dotx;
        let dot0a = (self.point * delta0 / denom - arrow.point * delta0 / denom).element_sum();
        let dot0b = (self.point * delta0 * dotx / denom - arrow.point * delta0 * dotx / denom).element_sum();
        let dot1a = (self.point * delta1 * dotx / denom - arrow.point * delta1 * dotx / denom).element_sum();
        let dot1b = (self.point * delta1 / denom - arrow.point * delta1 / denom).element_sum();
        // let dot1 = (self.point * delta1 - arrow.point * delta1).element_sum();
        let u0 = dot1a - dot0a;
        let u1 = dot1b - dot0b;
        let closest0 = self.point  + delta0 * u0;
        let closest1 = arrow.point + delta1 * u1;
        let point = (closest0 + closest1) / 2.;
        if point.is_nan() {
            //log("arrow.middle -> nan!");
            console_log!("arrow.middle point distance {}", self.point.distance(arrow.point));
            console_log!("self.delta, arrow.delta {}, {}", self.delta, arrow.delta);
            console_log!("delta0, delta1 {}, {}", delta0, delta1);
            panic!("arrow.middle -> nan!");
        }
        point
    }
}




pub trait ToCurve {
    fn to_curve(self) -> Shape;
}

impl ToCurve for Vec<Arrow> {
    fn to_curve(self) -> Shape {
        ArrowsToCurve {
            curve: Shape::from_order(3),
            arrow: self.first().expect(TWO_ARROWS).clone(),
            knot:  0.,
        }.make(self)
    }
}

pub struct ArrowsToCurve {
    curve: Shape,
    arrow: Arrow,
    knot:  f32,
}

impl ArrowsToCurve {
    fn make(&mut self, arrows: Vec<Arrow>) -> Shape {
        self.curve.controls.push(Shape::from_point(self.arrow.point));
        self.curve.basis.weights.push(1.);
        let delta = arrows.get(1).expect(TWO_ARROWS).delta;
        let mut base_angle = self.arrow.delta.angle_between(delta);
        for arrow in arrows.windows(2) {
            let angle = self.arrow.delta.angle_between(arrow[1].delta);
            if angle > FRAC_PI_8 || angle < base_angle - 0.001 {
                self.add_arc(&arrow[0]);
                base_angle = self.arrow.delta.angle_between(arrow[1].delta);
            }else{
                base_angle = angle;
            }
            self.knot += 1.; 
        }
        self.add_arc(&arrows.last().expect(TWO_ARROWS));
        self.curve.basis.knots.push(self.knot);
        self.curve.basis.normalize();
        self.curve.clone()
    }
    fn add_arc(&mut self, arrow: &Arrow) { 
        let middle = self.arrow.middle(arrow);
        self.curve.controls.push(Shape::from_point(middle)); 
        self.curve.controls.push(Shape::from_point(arrow.point));
        self.curve.basis.knots.extend(&[self.knot, self.knot]);
        let angle = self.arrow.delta.angle_between(arrow.delta);
        self.curve.basis.weights.extend(&[(angle / 2.).cos(), 1.]);  
        self.arrow = arrow.clone();
    }
}





// pub fn middle(&self, arrow: &Arrow) -> Vec3 {
//     let delta0 = self.delta.normalize();
//     let delta1 = arrow.delta.normalize();
//     let dotx = delta0.dot(delta1);
//     if dotx.abs() > DOT_1_TOL { 
//         return (self.point + arrow.point) / 2.;
//     }
//     let delta = self.point - arrow.point;
//     let dot0 = delta.dot(delta0);
//     let dot1 = delta.dot(delta1);
//     let denom = 1. - dotx * dotx;
//     let u0 = (       dotx * dot1 - dot0) / denom;
//     let u1 = (dot1 - dotx * dot0)        / denom;
//     let closest0 = self.point  + delta0 * u0;
//     let closest1 = arrow.point + delta1 * u1;
//     let point = (closest0 + closest1) / 2.;
//     if point.is_nan() {
//         //log("arrow.middle -> nan!");
//         console_log!("arrow.middle point distance {}", self.point.distance(arrow.point));
//         console_log!("self.delta, arrow.delta {}, {}", self.delta, arrow.delta);
//         console_log!("delta0, delta1 {}, {}", delta0, delta1);
//         panic!("arrow.middle -> nan!");
//     }
//     point
// }




// if (self.arrow.point - middle).length() == 0. {
//     log("self.arrow and middle the same");
//     console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
// }
// if (arrow.point - middle).length() == 0. {
//     log("arrow and middle the same");
//     console_log!("diff of self.arrow and arrow {}", (self.arrow.point - arrow.point).length());
// }




// impl Arrow {
//     pub fn new(point: Vec3, delta: Vec3) -> Self {
//         Self {point, delta}
//     }
//     pub fn middle(&self, arrow: &Arrow) -> Vec3 {
//         let d0 = self.delta.normalize();
//         let d1 = arrow.delta.normalize();
//         let b = d0.dot(d1);
//         if b.abs() > 0.99 { 
//             return (self.point + arrow.point) / 2.;
//         }
//         let a = d0.dot(d0);
//         let c = d1.dot(d1);
//         let delta = self.point - arrow.point;
//         let d = d0.dot(delta);
//         let e = d1.dot(delta);
//         let denom = a * c - b * b;
//         let u0 = (b * e - c * d) / denom;
//         let u1 = (a * e - b * d) / denom;
//         let closest0 = self.point  + d0 * u0;
//         let closest1 = arrow.point + d1 * u1;
//         let point = (closest0 + closest1) / 2.;
//         if point.is_nan() {
//             log("arrow.middle -> nan!");
//         }
//         point
//     }
// }




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