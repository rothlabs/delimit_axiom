use crate::shape::*;
use glam::*;
use super::ToRevolve;

pub fn sketch() -> Sketch {
    Sketch::default()
}

#[derive(Default)]
pub struct Sketch {
    pub shapes:  Vec<Shape>,
    pub start:   Vec2,
    turtle:  Turtle,
    pub drawing: bool,
    pub closed:  bool,
}

impl Sketch { 
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shapes = self.shapes.clone();
        if !self.closed {
            shapes.push(rank0(self.start.extend(0.)));
        }
        shapes 
    }
    pub fn jump_to(&mut self, point: Vec2) -> &mut Self {
        if self.drawing {
            self.shapes.push(rank0(self.start.extend(0.)));
        }
        self.turtle.jump_to(point);
        self.start = self.turtle.pos;
        self.drawing = false;
        self
    }
    pub fn jump_forward(&mut self, length: f32) -> &mut Self {
        if self.drawing {
            self.shapes.push(rank0(self.start.extend(0.)));
        }
        self.turtle.jump_forward(length);
        self.start = self.turtle.pos;
        self.drawing = false;
        self
    }
    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        let mut curve = Shape::default();
        curve.controls = vec![rank0(self.turtle.pos.extend(0.)), rank0(point.extend(0.))]; 
        curve.basis.order = 2;
        curve.validate();
        self.shapes.push(curve);
        self.shapes.push(rank0(point.extend(0.)));
        self.turtle.jump_to(point);
        self.drawing = true;
        self
    }
    pub fn line_forward(&mut self, length: f32) -> &mut Self {
        let start_point = self.turtle.pos;
        self.turtle.jump_forward(length);
        let end_point = self.turtle.pos;
        let mut curve = Shape::default();
        curve.controls = vec![rank0(start_point.extend(0.)), rank0(end_point.extend(0.))];
        curve.validate(); 
        self.shapes.push(curve);
        self.shapes.push(rank0(end_point.extend(0.)));
        self.drawing = true;
        self
    }
    pub fn turn(&mut self, angle: f32, radius: f32) -> &mut Self {
        let center = self.turtle.pos + self.turtle.dir.perp() * radius * angle.signum(); 
        if radius > 0. {
            self.shapes.extend(
                rank0(self.turtle.pos.extend(0.)).revolve()
                    .center(center.extend(0.))
                    .axis(vec3(0., 0., angle.signum()))
                    .angle(angle.abs())
                    .shapes()
            );
            self.drawing = true;
        }
        self.turtle.turn(center, angle);
        self
    }
    pub fn close(&mut self) -> &mut Self {
        self.line_to(self.start);
        self.drawing = false;
        self.closed = true;
        self
    }
}

#[derive(Default)]
struct Turtle {
    pos: Vec2,
    dir: Vec2,
}

impl Turtle {
    fn jump_to(&mut self, to: Vec2) {
        self.dir = (to - self.pos).normalize();
        self.pos = to;
    }
    fn jump_forward(&mut self, length: f32) {
        self.pos += self.dir * length;
    }
    fn turn(&mut self, center: Vec2, angle: f32) {
        let mat3 = Mat3::from_translation(center)
            * Mat3::from_axis_angle(Vec3::Z, angle)
            * Mat3::from_translation(-center);
        self.pos = mat3.mul_vec3(self.pos.extend(1.)).xy();
        self.dir = Vec2::from_angle(self.dir.to_angle() + angle);
    }
}






// pub trait ToSketch {
//     fn sketch(self) -> SketchActor;
// }

// impl ToSketch for Vec<Shape> {
//     fn sketch(self) -> SketchActor {
//         SketchActor{
//             shapes: self,
//             ..Default::default()
//         }
//     }
// }






            // let revolve = actor::Revolve {
            //     shapes: vec![rank0(self.turtle.pos.extend(0.))], 
            //     center: center.extend(0.),
            //     axis:   vec3(0., 0., angle.signum()),
            //     angle:  angle.abs(),
            //     ..Default::default()
            // };



// let mut shapes = actor::Revolve {
//     shapes: vec![rank0(vec3(radius, 0., 0.))], 
//     ..Default::default()
// }.shapes();