use crate::{Shape, Model, ModelsToShapes, Reshape, Shapes};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "GridPattern::default")]
pub struct GridPattern {
    pub parts:    Vec<Model>,
    pub reshape:  Reshape,
    pub count:    [usize; 3],
    pub length:   [f32; 3],
    pub x_count:  usize,
    pub y_count:  usize,
    pub z_count:  usize,
    pub x_length: f32,
    pub y_length: f32,
    pub z_length: f32,
}

impl GridPattern {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut x_count = self.x_count;
        let mut y_count = self.y_count;
        let mut z_count = self.z_count;
        if self.count[0] > 0 {x_count = self.count[0];}
        if self.count[1] > 0 {y_count = self.count[1];}
        if self.count[2] > 0 {z_count = self.count[2];}
        x_count = x_count.max(1);
        y_count = y_count.max(1);
        z_count = z_count.max(1);
        let mut x_length = self.x_length;
        let mut y_length = self.y_length;
        let mut z_length = self.z_length;
        if self.length[0] > 0. {x_length = self.length[0];}
        if self.length[1] > 0. {y_length = self.length[1];}
        if self.length[2] > 0. {z_length = self.length[2];}
        let basis = GridPatternBasis {
            count: [x_count, y_count, z_count],
            length: vec3(x_length, y_length, z_length),
        };
        self.reshape.get_reshapes(basis.get_shapes(self.parts.shapes()))
    }
}

pub struct GridPatternBasis {
    pub count:  [usize; 3],
    pub length: Vec3,
}

impl GridPatternBasis {
    pub fn get_shapes(&self, parts: Vec<Shape>) -> Vec<Shape> {
        let mut shapes = vec![];
        let mut div = (1., 1., 1.);
        if self.count[0] > 1 {div.0 = (self.count[0]-1) as f32;} 
        if self.count[1] > 1 {div.1 = (self.count[1]-1) as f32;} 
        if self.count[2] > 1 {div.2 = (self.count[2]-1) as f32;} 
        for x in 0..self.count[0] {
            for y in 0..self.count[1] {
                for z in 0..self.count[2] {
                    let pos = vec3(
                        (x as f32 / div.0) * self.length.x - self.length.x/2., 
                        (y as f32 / div.1) * self.length.y - self.length.y/2., 
                        (z as f32 / div.2) * self.length.z - self.length.z/2., 
                    );
                    let mat4 = Mat4::from_translation(pos);
                    shapes.extend(parts.reshaped(mat4));
                }
            }
        }
        shapes
    }
}


// for x in 0..x_count {
        //     for y in 0..y_count {
        //         for z in 0..z_count {
        //             let pos = vec3(
        //                 (x as f32 / x_div)*x_length - x_length/2., 
        //                 (y as f32 / y_div)*y_length - y_length/2., 
        //                 (z as f32 / z_div)*z_length - z_length/2., 
        //             );
        //             let mat4 = Mat4::from_translation(pos);
        //             shapes.extend(get_reshapes(&self.parts, mat4));
        //         }
        //     }
        // }
