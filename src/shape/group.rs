use glam::*;
use super::Shape;

pub trait Shapes {
    fn of_rank(&self, rank: u8) -> Vec<&Shape>;
    fn high_rank(&self) -> u8;
    fn reshape(&mut self, mat4: Mat4) -> &mut Self;
    fn reshaped(&self, mat4: Mat4) -> Vec<Shape>;
    fn reverse_direction(&mut self) -> &mut Self;
    //fn translate(&mut self) -> &mut Self;
}

impl Shapes for Vec<Shape> {
    fn of_rank(&self, rank: u8) -> Vec<&Shape> {
        let mut shapes = vec![];
        for shape in self {
            if shape.rank == rank {
                shapes.push(shape);   
            }
        }
        shapes
    }
    fn high_rank(&self) -> u8 {
        let mut rank = 0;
        for shape in self {
            rank = rank.max(shape.rank);
        }
        rank
    }
    fn reshape(&mut self, mat4: Mat4) -> &mut Self {
        for i in 0..self.len() {
            self[i].reshape(mat4);
        }
        self
    }
    fn reshaped(&self, mat4: Mat4) -> Vec<Shape> {
        let mut shapes = vec![];
        for shape in self {
            shapes.push(shape.reshaped(mat4));
        }
        shapes
    }
    fn reverse_direction(&mut self) -> &mut Self {
        for i in 0..self.len() {
            self[i].reverse();
        }
        self
    }
    // fn translate(&mut self, pos: Vec2) -> &mut Self {
        
    // }
}


pub trait Groups {
    fn negated(&self) -> Vec<Vec<Shape>>;
}

impl Groups for Vec<Vec<Shape>> {
    fn negated(&self) -> Vec<Vec<Shape>> {
        let mut groups = vec![];
        for group in self {
            let mut shapes = vec![];
            for shape in group {
                shapes.push(shape.negated());
            }
            groups.push(shapes);
        }
        groups
    }
}

pub trait Jobs {
    fn high_rank(&self) -> u8;
}

impl Jobs for Vec<Vec<Vec<Shape>>> {
    fn high_rank(&self) -> u8 {
        let mut rank = 0;
        for groups in self {
            for shapes in groups {
                for shape in shapes {
                    rank = rank.max(shape.rank);
                }
            }
        }
        rank
    }
}