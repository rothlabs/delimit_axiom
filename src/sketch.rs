use std::f32::consts::{FRAC_PI_2, PI};
use crate::{get_shapes, Curve, Reshape, Model, Revolve, Shape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Sketch {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
    pub actions: Vec<Action>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    JumpTo([f32; 2]),
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
            reshape: self.reshape.clone(),
            actions: self.actions.clone(),
            start_point: vec2(0., 0.),
            turtle: Turtle::default(),
        };
        sketch_shape.build_from_actions() 
    }
}

#[derive(Default)]
pub struct SketchShape {
    pub shapes:  Vec<Shape>,
    pub actions: Vec<Action>,
    pub start_point: Vec2,
    pub turtle: Turtle,
    pub reshape: Reshape,
}

impl SketchShape { 
    pub fn get_shapes(&self) -> Vec<Shape> {
        self.reshape.get_reshapes(self.shapes.clone())
    }
    pub fn build_from_actions(&mut self) -> Vec<Shape> {
        let mut closed = false;
        for action in self.actions.clone() {
            match action {
                Action::JumpTo(p)  => self.jump_to(Vec2::from_array(p)),
                Action::LineTo(p)  => self.line_to(Vec2::from_array(p)),
                Action::Turn(turn) => self.turn(turn.angle, turn.radius),
                Action::Close(_)   => {
                    closed = true;
                    self.close()
                },
            };
        }
        if !closed {
            self.shapes.push(Shape::Point(self.start_point.extend(0.)));
        }
        self.reshape.get_reshapes(self.shapes.clone())
    }
    fn jump_to(&mut self, point: Vec2) -> &mut Self {
        self.start_point = point;
        self.turtle.jump_to(point);
        self
        //self.actions.push(Action::Start([x, y]));
    }
    fn line_to(&mut self, point: Vec2) -> &mut Self {
        let mut curve = Curve::default();
        // curve.nurbs.knots = vec![0., 0., 1., 1.];
        // curve.nurbs.weights = vec![1., 1.];
        curve.controls = vec![self.turtle.pos.extend(0.), point.extend(0.)]; 
        self.shapes.push(Shape::Curve(curve.get_valid()));
        self.shapes.push(Shape::Point(point.extend(0.)));
        self.turtle.jump_to(point);
        self
        //self.actions.push(Action::LineTo([x, y]));
    }
    fn jump_forward(&mut self, length: f32) -> &mut Self {
        self.turtle.jump_forward(length);
        self
    }
    fn line_forward(&mut self, length: f32) -> &mut Self {
        let start_point = self.turtle.pos;
        self.turtle.jump_forward(length);
        let end_point = self.turtle.pos;
        let mut curve = Curve::default();
        // curve.nurbs.knots = vec![0., 0., 1., 1.];
        // curve.nurbs.weights = vec![1., 1.];
        curve.controls = vec![start_point.extend(0.), end_point.extend(0.)]; 
        self.shapes.push(Shape::Curve(curve.get_valid()));
        self.shapes.push(Shape::Point(end_point.extend(0.)));
        self
    }
    fn turn(&mut self, angle: f32, radius: f32) -> &mut Self {
        let center = self.turtle.pos + self.turtle.dir.perp() * radius * angle.signum(); 
        if radius > 0. {
            let revolve = Revolve {
                parts: vec![Model::Point(self.turtle.pos.extend(0.))], // [self.turtle.pos.x, self.turtle.pos.y, 0.]
                center: center.extend(0.),//[center.x, center.y, 0.],
                axis: vec3(0., 0., angle.signum()),//[0., 0., angle.signum()],
                angle: angle.abs(),
                reshape: Reshape::default(),
            };
            self.shapes.extend(revolve.get_shapes());
        }
        self.turtle.turn(center, angle);
        self
        //self.actions.push(Action::Turn(Turn{angle, radius}));
    }
    fn close(&mut self) -> &mut Self {
        self.line_to(self.start_point);
        self
        //self.actions.push(Action::Close(true));
    }
}

#[derive(Default)]
pub struct Turtle {
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
        self.pos = mat3.mul_vec3(self.pos.extend(1.)).truncate();
        self.dir = Vec2::from_angle(self.dir.to_angle() + angle);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center:  Vec2,//[f32; 2], 
    pub radius:  f32,
    pub reverse: bool,
}

impl Circle {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut revolve = Revolve {
            parts: vec![Model::Point(vec3(self.center.x + self.radius, self.center.y, 0.))], // [self.center[0] + self.radius, self.center[1], 0.]
            center: self.center.extend(0.),//[self.center[0], self.center[1], 0.],
            axis: Vec3::Z,//[0., 0., 1.],
            angle: PI*2.,
            reshape: Reshape::default(),
        };
        revolve.reshape.reverse = self.reverse;
        revolve.get_shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub half_lengths: [f32; 2],
    pub lengths:      [f32; 2],
    pub point_a:   [f32; 2], 
    pub point_b:   [f32; 2], 
    pub radius:    f32,
    pub reverse:   bool,
}

impl Rectangle {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut sketch = SketchShape::default();
        sketch.reshape.reverse = self.reverse;
        let mut point_a = -Vec2::from_array(self.half_lengths);
        let mut point_b = -point_a;
        if self.lengths[0] > 0. || self.lengths[1] > 0. {
            point_a = -Vec2::from_array(self.lengths) / 2.;
            point_b = -point_a;
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        sketch.jump_to(point_a + Vec2::X * self.radius) // point_a[0]+self.radius, point_a[1]
            .line_to(vec2(point_b.x-self.radius, point_a.y)) // point_b[0]-self.radius, point_a[1]
            .turn(FRAC_PI_2, self.radius) // if self.radius > 0. {
            .line_to(point_b - Vec2::Y * self.radius) // point_b[0], point_b[1]-self.radius
            .turn(FRAC_PI_2, self.radius)
            .line_to(vec2(point_a.x+self.radius, point_b.y)) // point_a[0]+self.radius, point_b[1]
            .turn(FRAC_PI_2, self.radius)
            .line_to(point_a + Vec2::Y * self.radius)  // point_a[0], point_a[1]+self.radius
            .turn(FRAC_PI_2, self.radius)
            .get_shapes()
    }
    pub fn unit() -> Vec<Curve> {
        let mut curves = vec![];
        let mut rect = Rectangle::default();
        rect.point_a = [0., 0.];
        rect.point_b = [1., 1.];
        for shape in rect.get_shapes() {
            if let Shape::Curve(c) = shape {curves.push(c);}
        }
        curves
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
        sketch.reshape.reverse = self.reverse;
        let mut point_a = vec2(-self.half_length, 0.);
        let mut point_b = vec2( self.half_length, 0.);
        if self.length > 0. {
            point_a = vec2(-self.length/2., 0.);
            point_b = vec2(self.length/2., 0.);
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        sketch.jump_to(point_b).jump_to(point_a).turn(FRAC_PI_2, 0.)
            .jump_forward(self.radius)
            .turn(FRAC_PI_2, 0.)
            .line_forward((point_a-point_b).length())
            .turn(PI, self.radius)
            .line_forward((point_a-point_b).length())
            .turn(PI, self.radius)
            .get_shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Arc::default")]
pub struct Arc {
    pub center: [f32; 2], 
    pub radius: f32,
    pub angle_a: f32,
    pub angle_b: f32,
    pub point_a: f32,
    pub point_b: f32,
    pub point_c: f32,
    //pub reverse: bool,
}

impl Arc {
    pub fn get_shapes(&self) -> Vec<Shape> {
        // let mut revolve = Revolve {
        //     parts: vec![Model::Point([self.center[0] + self.radius, self.center[1], 0.])],
        //     center: [self.center[0], self.center[1], 0.],
        //     axis: [0., 0., 1.],
        //     angle: self.angle,
        //     transform: Group::default(),
        // };
        // //revolve.transform.reverse = self.reverse;
        // revolve.get_shapes()
        vec![]
    }
}



                    // let mut curve = CurveShape::default();
                    // curve.nurbs.knots = vec![0., 0., 1., 1.];
                    // curve.nurbs.weights = vec![1., 1.];
                    // curve.controls = vec![vec3(turtle.pos.x, turtle.pos.y, 0.), vec3(p[0], p[1], 0.)]; 
                    // shapes.push(Shape::Curve(curve));
                    // shapes.push(Shape::Point(vec3(p[0], p[1], 0.)));
                    // turtle.jump_to(p[0], p[1]);