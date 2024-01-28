//use std::ops::{Add, Mul};

use super::{Model, polyline::*};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Turtled::default")]
pub struct Turtled {
    pub actions: Vec<Action>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Position(Vec<f32>),
    Forward(f32),
    TurnArc(TurnArc),
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "TurnArc::default")]
pub struct TurnArc {
    pub angle: f32,
    pub radius: f32,
}

impl Polyline for Turtled {
    fn get_polyline(&self, count: usize) -> Vec<f32> {
        let mut turtle = Turtle::new();
        for action in self.actions.clone(){
            match action {
                Action::Position(pos) => turtle.set_position(pos),
                Action::Forward(dist) => turtle.move_forward(dist),
                Action::TurnArc(turn) => turtle.turn_over_arc(turn, count),
            };
        }
        turtle.polyline
    }
}

struct Turtle {
    position:  Vec3,
    direction: Vec3,
    normal:    Vec3,
    polyline:  Vec<f32>,
}

impl Turtle {
    fn new() -> Turtle {
        Turtle {
            position:  Vec3::ZERO, 
            direction: Vec3::X, 
            normal:    Vec3::Z,
            polyline:  vec![],
        }
    }
    fn set_position(&mut self, pos: Vec<f32>) -> &mut Self {
        self.position = Vec3::new(pos[0], pos[1], pos[2]);
        self.polyline.extend(pos);
        self
    }
    fn move_forward(&mut self, dist: f32) -> &mut Self {
        self.position += self.direction * dist;
        self.polyline.extend(self.position.to_array());
        self
    }
    fn turn_over_arc(&mut self, turn: TurnArc, count: usize) -> &mut Self {
        let cross = self.direction.cross(self.normal);
        let pivot = self.position - cross * turn.radius;
        let mut swing = cross * turn.radius;
        let rotation = Mat3::from_axis_angle(self.normal, turn.angle.to_radians() / count as f32);
        for _ in 0..count {
            self.direction = rotation.mul_vec3(self.direction);
            swing = rotation.mul_vec3(swing);
            self.position = pivot + swing;
            self.polyline.extend(self.position.to_array());
        }
        self
    }
}