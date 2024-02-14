use std::f32::consts::{FRAC_PI_2, PI};
use crate::{Model, CurveShape, Revolve, Shape, log};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Sketch::default")]
pub struct Sketch {
    pub actions: Vec<Action>,
    //pub reverse: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    MoveTo([f32; 2]),
    LineTo([f32; 2]),
    Turn(Turn),
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Turn::default")]
pub struct Turn {
    pub angle:  f32,
    pub radius: f32,
}

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

impl Sketch { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let mut turtle = Turtle::default();
        for action in &self.actions {
            match action {
                Action::MoveTo(p) => {
                    turtle.move_to(p[0], p[1]);
                },
                Action::LineTo(p) => {
                    let mut curve = CurveShape::default();
                    curve.knots = vec![0., 0., 1., 1.];
                    curve.weights = vec![1., 1.];
                    curve.controls = vec![[turtle.pos.x, turtle.pos.y, 0.], [p[0], p[1], 0.]]; 
                    shapes.push(Shape::Curve(curve));
                    shapes.push(Shape::Point([p[0], p[1], 0.]));
                    turtle.move_to(p[0], p[1]);
                },
                Action::Turn(turn) => {
                    let center = turtle.pos + turtle.dir.perp() * turn.radius * turn.angle.signum(); 
                    let revolve = Revolve {
                        parts: vec![Model::Point([turtle.pos.x, turtle.pos.y, 0.])],
                        center: [center.x, center.y, 0.],
                        axis: [0., 0., turn.angle.signum()],
                        angle: turn.angle.abs(),
                    };
                    shapes.extend(revolve.get_shapes());
                    turtle.turn(center, turn.angle);
                },
            }
        }
        shapes
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.actions.push(Action::MoveTo([x, y]));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.actions.push(Action::LineTo([x, y]));
    }
    fn turn(&mut self, angle: f32, radius: f32) {
        self.actions.push(Action::Turn(Turn{angle, radius}));
    }
}

#[derive(Default)]
struct Turtle {
    pos: Vec2,
    dir: Vec2,
}

impl Turtle {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = vec2(x, y);
        self.dir = (to - self.pos).normalize();
        self.pos = to;
    }
    fn turn(&mut self, center: Vec2, angle: f32) {
        let mat3 = Mat3::from_translation(center)
            * Mat3::from_axis_angle(Vec3::Z, angle)
            * Mat3::from_translation(-center);
        self.pos = mat3.mul_vec3(self.pos.extend(1.)).truncate();
        self.dir = Vec2::from_angle(self.dir.to_angle() + angle);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Arc::default")]
pub struct Arc {
    pub center: [f32; 2], 
    pub radius: f32,
    pub angle: f32,
}

impl Arc {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut angle = self.angle;
        if angle == 0. {angle = PI*2.;}
        let revolve = Revolve {
            parts: vec![Model::Point([self.center[0] + self.radius, self.center[1], 0.])],
            center: [self.center[0], self.center[1], 0.],
            axis: [0., 0., 1.],
            angle,
        };
        revolve.get_shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    [f32; 2], 
    pub max:    [f32; 2], 
    pub radius: f32,
}

impl Rectangle {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut sketch = Sketch::default();
        sketch.move_to(self.min[0]+self.radius, self.min[1]);
        sketch.line_to(self.max[0]-self.radius, self.min[1]);
        sketch.turn(FRAC_PI_2, self.radius);
        sketch.line_to(self.max[0], self.max[1]-self.radius);
        sketch.turn(FRAC_PI_2, self.radius);
        sketch.line_to(self.min[0]+self.radius, self.max[1]);
        sketch.turn(FRAC_PI_2, self.radius);
        sketch.line_to(self.min[0], self.min[1]+self.radius);
        sketch.turn(FRAC_PI_2, self.radius);
        sketch.get_shapes()
    }
}
