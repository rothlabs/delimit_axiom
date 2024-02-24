use std::f32::consts::{FRAC_PI_2, PI};
use crate::{get_shapes, CurveShape, Group, Model, Revolve, Shape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Sketch::default")]
pub struct Sketch {
    pub parts:   Vec<Model>,
    pub actions: Vec<Action>,
    pub transform: Group,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Start([f32; 2]),
    LineTo([f32; 2]),
    Turn(Turn),
    Close(bool),
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Turn::default")]
pub struct Turn {
    pub angle:  f32,
    pub radius: f32,
}

impl Sketch {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut sketch_shape = SketchShape {
            shapes: get_shapes(&self.parts),
            actions: self.actions.clone(),
            start_point: vec2(0., 0.),
            turtle: Turtle::default(),
            transform: self.transform.clone(),
        };
        sketch_shape.build_shapes_from_actions() 
    }
}

#[derive(Default)]
pub struct SketchShape {
    pub shapes:  Vec<Shape>,
    pub actions: Vec<Action>,
    pub start_point: Vec2,
    pub turtle: Turtle,
    pub transform: Group,
}

impl SketchShape { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        self.transform.get_reshapes(self.shapes)
    }
    pub fn build_shapes_from_actions(&mut self) -> Vec<Shape> {
        //let mut shapes = get_shapes(&self.parts);
        //let mut turtle = Turtle::default();
        //let mut start_point = vec2(0., 0.);
        for action in self.actions.clone() {
            match action {
                Action::Start(p) => {
                    start_point = Vec2::from_array(p);
                    self.turtle.move_to(start_point);
                },
                Action::LineTo(p) => self.add_line(Vec2::from_array(p)),
                Action::Turn(turn) => {
                    let center = self.turtle.pos + self.turtle.dir.perp() * turn.radius * turn.angle.signum(); 
                    let revolve = Revolve {
                        parts: vec![Model::Point([self.turtle.pos.x, self.turtle.pos.y, 0.])],
                        center: [center.x, center.y, 0.],
                        axis: [0., 0., turn.angle.signum()],
                        angle: turn.angle.abs(),
                        transform: Group::default(),
                    };
                    self.shapes.extend(revolve.get_shapes());
                    self.turtle.turn(center, turn.angle);
                },
                Action::Close(_) => {
                    self.add_line(start_point);
                },
            }
        }
        self.transform.get_reshapes(self.shapes.clone())
    }
    fn add_line(&mut self, p: Vec2) {
        let mut curve = CurveShape::default();
        curve.nurbs.knots = vec![0., 0., 1., 1.];
        curve.nurbs.weights = vec![1., 1.];
        curve.controls = vec![vec3(self.turtle.pos.x, self.turtle.pos.y, 0.), p.extend(0.)]; 
        self.shapes.push(Shape::Curve(curve));
        self.shapes.push(Shape::Point(p.extend(0.)));
        self.turtle.move_to(p);
    }
    fn start(&mut self, x: f32, y: f32) {
        self.actions.push(Action::Start([x, y]));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.actions.push(Action::LineTo([x, y]));
    }
    fn turn(&mut self, angle: f32, radius: f32) {
        self.actions.push(Action::Turn(Turn{angle, radius}));
    }
    fn close(&mut self) {
        self.actions.push(Action::Close(true));
    }
}

#[derive(Default)]
pub struct Turtle {
    pos: Vec2,
    dir: Vec2,
}

impl Turtle {
    fn move_to(&mut self, to: Vec2) {
        //let to = vec2(x, y);
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
    fn forward(&mut self, length: f32) {
        self.pos += self.dir * length;
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Arc::default")]
pub struct Arc {
    pub center: [f32; 2], 
    pub radius: f32,
    pub angle: f32,
    pub reverse: bool,
}

impl Arc {
    pub fn get_shapes(&self) -> Vec<Shape> {
        //let angle = self.angle;
        //if angle == 0. {angle = PI*2.;}
        let mut revolve = Revolve {
            parts: vec![Model::Point([self.center[0] + self.radius, self.center[1], 0.])],
            center: [self.center[0], self.center[1], 0.],
            axis: [0., 0., 1.],
            angle: self.angle,
            transform: Group::default(),
        };
        revolve.transform.reverse = self.reverse;
        revolve.get_shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub half_length: [f32; 2],
    pub length:      [f32; 2],
    pub point_a:   [f32; 2], 
    pub point_b:   [f32; 2], 
    pub radius:    f32,
    pub reverse:   bool,
}

impl Rectangle {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut sketch = SketchShape::default();
        sketch.transform.reverse = self.reverse;
        let mut point_a = [-self.half_length[0], -self.half_length[1]];
        let mut point_b = [ self.half_length[0],  self.half_length[1]];
        if self.length[0] > 0. || self.length[1] > 0. {
            point_a = [-self.length[0]/2., -self.length[1]/2.];
            point_b = [ self.length[0]/2.,  self.length[1]/2.];
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = self.point_a;
            point_b = self.point_b;
        }
        sketch.start(point_a[0]+self.radius, point_a[1]);
        sketch.line_to(point_b[0]-self.radius, point_a[1]);
        if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        sketch.line_to(point_b[0], point_b[1]-self.radius);
        if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        sketch.line_to(point_a[0]+self.radius, point_b[1]);
        if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        sketch.line_to(point_a[0], point_a[1]+self.radius);
        if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        sketch.build()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Slot::default")]
pub struct Slot {
    pub length:      f32,
    pub half_length: f32,
    pub point_a:   [f32; 2], 
    pub point_b:   [f32; 2], 
    pub radius:    f32,
    pub reverse:   bool,
}

impl Slot {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut sketch = SketchShape::default();
        sketch.transform.reverse = self.reverse;
        let mut point_a = [-self.half_length, 0.];
        let mut point_b = [ self.half_length, 0.];
        if self.length > 0. {
            point_a = [-self.length/2., 0.];
            point_b = [ self.length/2., 0.];
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = self.point_a;
            point_b = self.point_b;
        }
        sketch.turtle.move_to(point_a[0], point_a[1]);
        // sketch.line_to(point_b[0]-self.radius, point_a[1]);
        // if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        // sketch.line_to(point_b[0], point_b[1]-self.radius);
        // if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        // sketch.line_to(point_a[0]+self.radius, point_b[1]);
        // if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        // sketch.line_to(point_a[0], point_a[1]+self.radius);
        // if self.radius > 0. {sketch.turn(FRAC_PI_2, self.radius);}
        sketch.build()
    }
}



                    // let mut curve = CurveShape::default();
                    // curve.nurbs.knots = vec![0., 0., 1., 1.];
                    // curve.nurbs.weights = vec![1., 1.];
                    // curve.controls = vec![vec3(turtle.pos.x, turtle.pos.y, 0.), vec3(p[0], p[1], 0.)]; 
                    // shapes.push(Shape::Curve(curve));
                    // shapes.push(Shape::Point(vec3(p[0], p[1], 0.)));
                    // turtle.move_to(p[0], p[1]);