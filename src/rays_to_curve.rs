use std::f32::consts::FRAC_PI_2;
use glam::*;
use crate::{Ray, CurveShape};

const TWO_RAYS: &str = "There should be two rays or more.";

struct RaysToCurve {
    curve: CurveShape,
    angle: f32,
    knot:  f32,
    ray:   Ray,
}

impl RaysToCurve {

    fn new(rays: Vec<Ray>) -> CurveShape {
        let ray = rays.first().expect(TWO_RAYS).clone();
        let vector = rays.get(1).expect(TWO_RAYS).vector;
        Self {
            curve: CurveShape::from_order(3),
            angle: ray.vector.angle_between(vector),
            knot:  0.,
            ray,
        }.make(rays)
    }

    fn make(&mut self, rays: Vec<Ray>) -> CurveShape {
        self.curve.controls.push(self.ray.origin);
        self.curve.nurbs.weights.push(1.);
        for (i, ray) in rays.windows(2).enumerate() {
            let angle = self.ray.vector.angle_between(ray[1].vector);
            if angle > FRAC_PI_2 || angle < self.angle { // quarter turn or inflection 
                self.add_arc(&ray[0]);
            }
            self.angle = angle;
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
        self.curve.nurbs.weights.extend(&[(self.angle / 2.).cos(), 1.]);
        self.ray = ray.clone();
    }

}

// #[derive(Default, Clone)]
// pub struct Arrow {
//     pub origin: Vec3,
//     pub vector: Vec3,
// }

// #[derive(Default, Clone)]
// pub struct Polyline {
//     pub arrows: Vec<Arrow>,
// }

// impl Polyline {
//     fn to_curve(&self) -> CurveShape {
//         let mut curve = CurveShape::default();
//         curve.nurbs.order = 3;
//         curve.nurbs.knots = vec![0., 0.];
//         let mut knot = 0.;
//         let arrow_base = self.arrows.get(0).expect(two_arrows);
//         let angle_base = arrow_base.vector.dot(
//             self.arrows.get(1).expect(two_arrows).vector
//         );
//         for arrow in self.arrows.iter().skip(2) {
//             let angle = arrow_base.vector.angle_between(arrow.vector);
//             if angle_base > angle { // curve inflection 
//                 //let weight = anl
//                 curve.controls.push(arrow_base.origin);
//                 curve.controls.push(arrow.origin);
//                 curve.nurbs.knots.extend(&[knot, knot+1., knot+2.]);
//                 curve.nurbs.weights.extend(&[1., 1., 1.]);
//                 knot += 2.;

//             }
//         }
//         curve.nurbs.knots.extend(&[knot, knot]);
//         curve
//     }
// }