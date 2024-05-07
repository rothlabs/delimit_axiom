use std::f32::consts::{PI, FRAC_PI_2};
use glam::*;
use crate::shape::*;
use super::ToRevolve;

pub fn circle() -> Circle {
    Circle {
        radius: 1.,
        center: Vec2::ZERO,
    }
}

pub struct Circle {
    pub radius: f32,
    pub center: Vec2,
}

impl Circle {
    pub fn shapes(&self) -> Vec<Shape> {
        rank0d3(self.center.x + self.radius, self.center.y, 0.).revolve()
            .center(self.center.extend(0.))
            .shapes()
    }
    pub fn center(&mut self, center: Vec2) -> &mut Self {
        self.center = center;
        self
    }
    pub fn radius(&mut self, radius: f32) -> &mut Self {
        self.radius = radius;
        self
    }
}

pub fn rectangle() -> Rectanlge {
    Rectanlge {
        points: (Vec2::ZERO, Vec2::ONE),
        anchor: Vec2::ZERO,
        size:   Vec2::ONE,
        radius: 0.,
    }
}

pub struct Rectanlge {
    pub points: (Vec2, Vec2),
    pub anchor: Vec2,
    pub size:   Vec2,
    pub radius: f32,
}

impl Rectanlge {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut points = self.points;
        if self.size.length() > 0. {
            let origin = -self.size * self.anchor;
            points.0 = origin;
            points.1 = origin + self.size;
        }
        super::sketch()
            .jump_to(points.0 + Vec2::X * self.radius) // point.0[0]+self.radius, point.0[1]
            .line_to(vec2(points.1.x-self.radius, points.0.y)) // point.1[0]-self.radius, point.0[1]
            .turn(FRAC_PI_2, self.radius) // if self.radius > 0. {
            .line_to(points.1 - Vec2::Y * self.radius) // point.1[0], point.1[1]-self.radius
            .turn(FRAC_PI_2, self.radius)
            .line_to(vec2(points.0.x+self.radius, points.1.y)) // point.0[0]+self.radius, point.1[1]
            .turn(FRAC_PI_2, self.radius)
            .line_to(points.0 + Vec2::Y * self.radius)  // point.0[0], point.0[1]+self.radius
            .turn(FRAC_PI_2, self.radius)
            .shapes()
    }
    pub fn points(&mut self, points: (Vec2, Vec2)) -> &mut Self {
        self.points = points;
        self
    }
    pub fn size(&mut self, size: Vec2) -> &mut Self {
        self.size = size;
        self
    }
    pub fn anchor(&mut self, anchor: Vec2) -> &mut Self {
        self.anchor = anchor;
        self
    }
    pub fn radius(&mut self, radius: f32) -> &mut Self {
        self.radius = radius;
        self
    }
}

pub fn slot() -> Slot {
    Slot {
        points: (-Vec2::X, Vec2::X),
        radius: 1.,
    }
}

pub struct Slot {
    pub points: (Vec2, Vec2),
    pub radius: f32,
}

impl Slot {
    pub fn shapes(&self) -> Vec<Shape> {
        super::sketch()
            .jump_to(self.points.1).jump_to(self.points.0).turn(FRAC_PI_2, 0.)
            .jump_forward(self.radius)
            .turn(FRAC_PI_2, 0.)
            .line_forward((self.points.0 - self.points.1).length())
            .turn(PI, self.radius)
            .line_forward((self.points.0 - self.points.1).length())
            .turn(PI, self.radius)
            .shapes()
    }
    pub fn points(&mut self, a: Vec2, b: Vec2) -> &mut Self {
        self.points.0 = a;
        self.points.1 = b;
        self
    }
    pub fn radius(&mut self, radius: f32) -> &mut Self {
        self.radius = radius;
        self
    }
}

