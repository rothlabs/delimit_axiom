use std::f32::consts::FRAC_PI_4;
use glam::*;
use crate::{CurveShape, Ray};

const TWO_RAYS: &str = "There should be two rays or more.";

pub struct RaysToCurve {
    curve: CurveShape,
    knot:  f32,
    ray:   Ray,
}

impl RaysToCurve {

    pub fn new(rays: Vec<Ray>) -> CurveShape {
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
            let angle = self.ray.vector.angle_between(ray[1].vector);
            if angle > FRAC_PI_4 || angle < base_angle { // 1/8th turn or inflection 
                self.add_arc(&ray[0]);
                base_angle = 0.;
            }else{
                base_angle = angle;
            }
            if i+3 > rays.len() {
                self.add_arc(&ray[1]);
            }
        }
        self.curve.nurbs.knots.push(self.knot);
        self.curve.clone()
    }

    fn add_arc(&mut self, ray: &Ray) {
        self.curve.controls.push(self.ray.middle(ray)); 
        self.curve.controls.push(ray.origin);
        self.knot += 1.;
        self.curve.nurbs.knots.extend(&[self.knot, self.knot]);
        let angle = self.ray.vector.angle_between(ray.vector);
        self.curve.nurbs.weights.extend(&[(angle / 2.).cos(), 1.]); 
        self.ray = ray.clone();
    }

}