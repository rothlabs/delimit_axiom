use std::f32::consts::FRAC_PI_4;
use glam::*;
use crate::{log, CurveShape, Ray};

const TWO_RAYS: &str = "There should be two rays or more.";

pub struct RaysToCurve {
    curve: CurveShape,
    knot:  f32,
    ray:   Ray,
}

impl RaysToCurve {

    pub fn new(rays: Vec<Ray>) -> CurveShape {
        //let wow = rays.get(2).expect("Should be at least three rays!");
        Self {
            curve: CurveShape::from_order(3),
            ray: rays.first().expect(TWO_RAYS).clone(),
            knot:  0.,
        }.make(rays)
    }

    fn make(&mut self, rays: Vec<Ray>) -> CurveShape {
        self.curve.controls.push(self.ray.origin);
        self.curve.nurbs.weights.push(1.);
        let vector = rays.get(1).expect(TWO_RAYS).vector;
        let mut base_angle = self.ray.vector.angle_between(vector);
        for (i, ray) in rays.windows(2).enumerate() {
            if i > 0 {
                let angle = self.ray.vector.angle_between(ray[1].vector);
                if angle > FRAC_PI_4 || (angle > 0.01 && angle < base_angle) { // 1/8th turn or inflection 
                    self.add_arc(&ray[0], rays.len()-i-2);
                    base_angle = 0.;
                }else{
                    base_angle = angle;
                }
                if i+3 > rays.len() {
                    self.add_arc(&ray[1], rays.len()-i-2);
                }
            }
        }
        self.curve.nurbs.knots.push(self.knot);
        self.curve.clone()
    }

    fn add_arc(&mut self, ray: &Ray, remaining: usize) {
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
            self.curve.controls.push(self.ray.middle(ray)); 
            self.curve.controls.push(ray.origin);
            self.knot += (ray.origin - self.ray.origin).length();
            self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
            let angle = self.ray.vector.angle_between(ray.vector);
            self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]); 
            self.ray = ray.clone();
        //}
    }

}