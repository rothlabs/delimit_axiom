use std::f32::consts::{FRAC_PI_2, PI};
use crate::shape::*;
use crate::{actor, Model, Models, Reshape};
use serde::*;
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Sketch {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
    pub actions: Vec<Action>,
}

impl Sketch {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shapes = self.parts.shapes();
        let mut sketch = actor::Sketch::default();
        for action in &self.actions {
            match action {
                Action::JumpTo(p)  => sketch.jump_to(Vec2::from_array(*p)),
                Action::LineTo(p)  => sketch.line_to(Vec2::from_array(*p)),
                Action::Turn(turn) => sketch.turn(turn.angle, turn.radius),
                Action::Close(_)   => sketch.close(),
            };
        }
        shapes.extend(sketch.shapes());
        self.reshape.get_reshapes(shapes)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    JumpTo([f32; 2]),
    LineTo([f32; 2]),
    Turn(Turn),
    Close(bool),
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)] 
pub struct Turn {
    pub angle:  f32,
    pub radius: f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Circle {
    pub center:  Vec2, 
    pub radius:  f32,
    pub reverse: bool,
    pub arrows: usize,
}

impl Circle {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shapes = actor::Revolve {
            shapes: vec![rank0(vec3(self.center.x + self.radius, self.center.y, 0.))], 
            center: self.center.extend(0.),
            ..Default::default()
        }.shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
                    if self.arrows > 0 {
                        for i in 0..self.arrows {
                            let mut curve = Shape::default();
                            let arrow = shapes[0].get_arrow(&[i as f32 / (self.arrows - 1) as f32]);
                            curve.controls.push(rank0(arrow.point));
                            curve.controls.push(rank0(arrow.point + arrow.delta));
                            curve.validate();
                            shapes.push(curve);
                        }
                    }
        shapes
    }
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Rectangle {
    pub half_lengths: [f32; 2],
    pub lengths:      [f32; 2],
    pub point_a:   [f32; 2], 
    pub point_b:   [f32; 2], 
    pub radius:    f32,
    pub reverse:   bool,
}

impl Rectangle {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut sketch = actor::Sketch::default();
        let mut point_a = -Vec2::from_array(self.half_lengths);
        let mut point_b = -point_a;
        if self.lengths[0] > 0. || self.lengths[1] > 0. {
            point_a = -Vec2::from_array(self.lengths) / 2.;
            point_b = -point_a;
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        let mut shapes = sketch.jump_to(point_a + Vec2::X * self.radius) // point_a[0]+self.radius, point_a[1]
            .line_to(vec2(point_b.x-self.radius, point_a.y)) // point_b[0]-self.radius, point_a[1]
            .turn(FRAC_PI_2, self.radius) // if self.radius > 0. {
            .line_to(point_b - Vec2::Y * self.radius) // point_b[0], point_b[1]-self.radius
            .turn(FRAC_PI_2, self.radius)
            .line_to(vec2(point_a.x+self.radius, point_b.y)) // point_a[0]+self.radius, point_b[1]
            .turn(FRAC_PI_2, self.radius)
            .line_to(point_a + Vec2::Y * self.radius)  // point_a[0], point_a[1]+self.radius
            .turn(FRAC_PI_2, self.radius)
            .shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
        shapes
    }
    pub fn unit() -> Vec<Shape> {
        let mut curves = vec![];
        let mut rect = Rectangle::default();
        rect.point_a = [0., 0.];
        rect.point_b = [1., 1.];
        for shape in rect.shapes() { // TODO: is it needed to check for curves in the rectangle?
            if shape.rank == 1 {
                curves.push(shape);
            }
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
    pub fn shapes(&self) -> Vec<Shape> {
        let mut sketch = actor::Sketch::default();
        let mut point_a = vec2(-self.half_length, 0.);
        let mut point_b = vec2( self.half_length, 0.);
        if self.length > 0. {
            point_a = vec2(-self.length/2., 0.);
            point_b = vec2(self.length/2., 0.);
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        let mut shapes = sketch.jump_to(point_b).jump_to(point_a).turn(FRAC_PI_2, 0.)
            .jump_forward(self.radius)
            .turn(FRAC_PI_2, 0.)
            .line_forward((point_a-point_b).length())
            .turn(PI, self.radius)
            .line_forward((point_a-point_b).length())
            .turn(PI, self.radius)
            .shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
        shapes
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
    pub fn shapes(&self) -> Vec<Shape> {
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



        // let mut sketch = crate::SketchActor {
        //     shapes: self.parts.shapes(),
        //     //reshape: self.reshape.clone(),
        //     start: vec2(0., 0.),
        //     turtle: Turtle::default(),
        //     drawing: false,
        //     closed: false,
        // };