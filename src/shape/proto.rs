use crate::actor::ToRevolve;
use crate::shape::*;

pub fn circle(radius: f32) -> Vec<Shape> {
    rank0d3(radius, 0., 0.).revolve().shapes()
}