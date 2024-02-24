use crate::{get_reshapes, get_vec3_or, Group, Model, Shape};
use serde::{Deserialize, Serialize};
use glam::*;


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Pattern::default")]
pub struct Pattern {
    pub parts:    Vec<Model>,
    pub transform:  Group,
    pub grid:     Grid,
    pub x_count:  usize,
    pub y_count:  usize,
    pub z_count:  usize,
    pub x_length: f32,
    pub y_length: f32,
    pub z_length: f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Grid::default")]
pub struct Grid {
    pub count:  [usize; 3],
    pub length: [f32; 3],
}

impl Pattern {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut shapes = vec![];
        let mut x_count = self.x_count;
        let mut y_count = self.y_count;
        let mut z_count = self.z_count;
        if self.grid.count[0] > 0 {x_count = self.grid.count[0];}
        if self.grid.count[1] > 0 {y_count = self.grid.count[1];}
        if self.grid.count[2] > 0 {z_count = self.grid.count[2];}
        x_count = x_count.max(1);
        y_count = y_count.max(1);
        z_count = z_count.max(1);
        let mut x_div = 1.;
        let mut y_div = 1.;
        let mut z_div = 1.;
        if x_count > 1 {x_div = (x_count-1) as f32;} 
        if y_count > 1 {y_div = (y_count-1) as f32;} 
        if z_count > 1 {z_div = (z_count-1) as f32;} 
        let mut x_length = self.x_length;
        let mut y_length = self.y_length;
        let mut z_length = self.z_length;
        if self.grid.length[0] > 0. {x_length = self.grid.length[0];}
        if self.grid.length[1] > 0. {y_length = self.grid.length[1];}
        if self.grid.length[2] > 0. {z_length = self.grid.length[2];}
        for x in 0..x_count {
            for y in 0..y_count {
                for z in 0..z_count {
                    let pos = vec3(
                        (x as f32 / x_div)*x_length - x_length/2., 
                        (y as f32 / y_div)*y_length - y_length/2., 
                        (z as f32 / z_div)*z_length - z_length/2., 
                    );
                    let mat4 = Mat4::from_translation(pos);
                    shapes.extend(get_reshapes(&self.parts, mat4));
                }
            }
        }
        self.transform.get_reshapes(shapes)
    }
}

