use std::f32::consts::FRAC_PI_4;
use glam::*;
use crate::{log, CurveShape};

const TWO_RAYS: &str = "There should be two rays or more.";

#[derive(Default, Clone)]
pub struct Ray {
    pub origin:    Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, vector: Vec3) -> Self {
        Self {origin, direction: vector}
    }
    pub fn middle(&self, ray: &Ray) -> Vec3 {
        if self.direction.normalize().dot(ray.direction.normalize()) > 0.95 { // parallel case
            return (self.origin + ray.origin) / 2.;
        }
        let a = self.direction.dot(self.direction);
        let b = self.direction.dot(ray.direction);
        let c =  ray.direction.dot(ray.direction);
        let delta = self.origin - ray.origin;
        let d = self.direction.dot(delta);
        let e =  ray.direction.dot(delta);
        let denom = a * c - b * b;
        let u0 = (b * e - c * d) / denom;
        let u1 = (a * e - b * d) / denom;
        let p0 = self.origin + self.direction * u0;
        let p1 = ray.origin  + ray.direction  * u1;
        (p0 + p1) / 2.
    }
}


pub trait ToCurve {
    fn to_curve(self) -> CurveShape;
}

impl ToCurve for Vec<Ray> {
    fn to_curve(self) -> CurveShape {
        RaysToCurve {
            curve: CurveShape::from_order(3),
            ray: self.first().expect(TWO_RAYS).clone(),
            knot:  0.,
        }.make(self)
    }
}

pub struct RaysToCurve {
    curve: CurveShape,
    knot:  f32,
    ray:   Ray,
}

impl RaysToCurve {
    fn make(&mut self, rays: Vec<Ray>) -> CurveShape {
        self.curve.controls.push(self.ray.origin);
        self.curve.nurbs.weights.push(1.);
        let vector = rays.get(1).expect(TWO_RAYS).direction;
        let mut base_angle = self.ray.direction.angle_between(vector);
        for (i, ray) in rays.windows(2).enumerate() {
            if i > 0 {
                let angle = self.ray.direction.angle_between(ray[1].direction);
                if angle > FRAC_PI_4 || (angle > 0.01 && angle < base_angle) { // 1/8th turn or inflection 
                    self.add_arc(&ray[0]);
                    base_angle = 0.;
                }else{
                    base_angle = angle;
                }
                if i+3 > rays.len() {
                    self.add_arc(&ray[1]);
                }
            }
        }
        self.curve.nurbs.knots.push(self.knot);
        self.curve.nurbs.normalize_knots();
        self.curve.clone()
    }
    fn add_arc(&mut self, ray: &Ray) { 
        let middle = self.ray.middle(ray);
        self.curve.controls.push(middle); 
        self.curve.controls.push(ray.origin);
        if (self.ray.origin - middle).length() == 0. {
            log("self.ray and middle the same");
            console_log!("diff of self.ray and ray {}", (self.ray.origin - ray.origin).length());
        }
        if (ray.origin - middle).length() == 0. {
            log("ray and middle the same");
            console_log!("diff of self.ray and ray {}", (self.ray.origin - ray.origin).length());
        }
        self.knot += (ray.origin - self.ray.origin).length();
        //self.knot += 1.;
        self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
        let angle = self.ray.direction.angle_between(ray.direction);
        self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]); 
        self.ray = ray.clone();
    }
}


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