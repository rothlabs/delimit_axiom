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
        let mut sketch = actor::sketch();
        for action in &self.actions {
            match action {
                Action::JumpTo(p)  => sketch.jump_to(Vec2::from_array(*p)),
                Action::LineTo(p)  => sketch.line_to(Vec2::from_array(*p)),
                Action::Turn(turn) => sketch.turn(turn.angle, turn.radius),
                Action::Close(_)   => sketch.close(),
            };
        }
        shapes.extend(sketch.shapes());
        self.reshape.shapes(shapes)
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






        // let mut sketch = crate::SketchActor {
        //     shapes: self.parts.shapes(),
        //     //reshape: self.reshape.clone(),
        //     start: vec2(0., 0.),
        //     turtle: Turtle::default(),
        //     drawing: false,
        //     closed: false,
        // };



                //let mut shapes = circle(self.radius)
        //    .reshaped(Mat4::from_translation(self.center.extend(0.)));
        // let mut shapes = actor::Revolve {
        //     shapes: vec![rank0(vec3(self.center.x + self.radius, self.center.y, 0.))], 
        //     center: self.center.extend(0.),
        //     ..Default::default()
        // }.shapes();




        // let mut shapes = sketch.jump_to(point_a + Vec2::X * self.radius) // point_a[0]+self.radius, point_a[1]
        //     .line_to(vec2(point_b.x-self.radius, point_a.y)) // point_b[0]-self.radius, point_a[1]
        //     .turn(FRAC_PI_2, self.radius) // if self.radius > 0. {
        //     .line_to(point_b - Vec2::Y * self.radius) // point_b[0], point_b[1]-self.radius
        //     .turn(FRAC_PI_2, self.radius)
        //     .line_to(vec2(point_a.x+self.radius, point_b.y)) // point_a[0]+self.radius, point_b[1]
        //     .turn(FRAC_PI_2, self.radius)
        //     .line_to(point_a + Vec2::Y * self.radius)  // point_a[0], point_a[1]+self.radius
        //     .turn(FRAC_PI_2, self.radius)
        //     .shapes();



            // pub fn unit() -> Vec<Shape> {
    //     let mut curves = vec![];
    //     let mut rect = Rectangle::default();
    //     rect.point_a = [0., 0.];
    //     rect.point_b = [1., 1.];
    //     for shape in rect.shapes() { // TODO: is it needed to check for curves in the rectangle?
    //         if shape.rank == 1 {
    //             curves.push(shape);
    //         }
    //     }
    //     curves
    // }