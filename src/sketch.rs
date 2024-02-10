use crate::{Model, Nurbs, Revolve, Shape, log};
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

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

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
                    let mut nurbs = Nurbs::default();
                    nurbs.controls = vec![Shape::Point([turtle.pos.x, turtle.pos.y, 0.]), Shape::Point([p[0], p[1], 0.])]; 
                    shapes.push(Shape::Curve(nurbs));
                    shapes.push(Shape::Point([p[0], p[1], 0.]));
                    turtle.move_to(p[0], p[1]);
                },
                Action::Turn(turn) => {
                    let center = turtle.pos + turtle.dir.perp() * turn.radius * turn.angle.signum(); 
                    //console_log!("turtle X: {}", turtle.pos.x);
                    //console_log!("turtle Y: {}", turtle.pos.y);
                    let revolve = Revolve {
                        parts: vec![Model::Point([turtle.pos.x, turtle.pos.y, 0.])],
                        center: [center.x, center.y, 0.],
                        axis: [0., 0., turn.angle.signum()],
                        angle: turn.angle.abs(),
                    };
                    let r_shapes = revolve.get_shapes();
                    console_log!("r shapes: {}", r_shapes.len());
                    shapes.extend(r_shapes);
                    turtle.turn(center, turn.angle);
                },
            }
        }
        shapes
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
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center: Box<Model>, 
    pub radius: f32,
}

// impl Circle {
//     pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
//         get_polyline(self.get_path(), tolerance)
//     }
//     pub fn get_path(&self) -> lyon::path::Path {
//         let center = get_point(&*self.center);
//         let mut builder = lyon::path::Path::builder();
//         builder.add_circle(center, self.radius, Winding::Positive); 
//         builder.build()
//     }
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    Box<Model>, 
    pub max:    Box<Model>, 
    pub radius: f32,
}

// impl Rectangle {
//     pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
//         get_polyline(self.get_path(), tolerance)
//     }
//     pub fn get_path(&self) -> lyon::path::Path { 
//         let min = get_point(&*self.min);
//         let max = get_point(&*self.max);
//         let mut builder = lyon::path::Path::builder();
//         builder.add_rounded_rectangle(
//             &Box2D {min, max},
//             &BorderRadii {
//                 top_left:     self.radius,
//                 top_right:    self.radius,
//                 bottom_left:  self.radius,
//                 bottom_right: self.radius,
//             },
//             Winding::Positive,
//         );
//         builder.build()
//     }
// }

// fn get_point(model: &Model) -> Point {
//     match model {
//         Model::Vector(m) => point(m[0], m[1]),
//         _ => point(0., 0.),
//     }
// }

// fn get_point2(model: &Model) -> Vec<f32> {
//     match model {
//         Model::Vector(m) => vec![m[0], m[1], 0.],
//         _ => vec![0.; 3],
//     }
// }

// fn get_vector(model: &Model) -> Vector {
//     match model {
//         Model::Vector(m) => vector(m[0], m[1]),
//         _ => vector(0., 0.),
//     }
// }
