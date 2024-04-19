use std::collections::HashMap;
use glam::*;

use crate::DUP_TOL;

#[derive(Default)]
pub struct Spatial3 {
    pub map: HashMap<String, Vec<usize>>,
    cell_size: f32,
}

impl Spatial3 {
    pub fn new() -> Spatial3 {
        let map: HashMap<String, Vec<usize>> = HashMap::new();
        Spatial3 {
            map,
            cell_size: DUP_TOL,
        }
    }

    pub fn for_pairs<F>(&self, func: &mut F) where F: FnMut(usize, usize) {
        let mut point = [0; 3];
        for (key0, indices0) in &self.map {
            for (i, string_int) in key0.split(",").enumerate() {
                point[i] = string_int.parse().expect("failed to parse key in spatial3");
            }
            for x in -1..2 {
                for y in -1..2 {
                    for z in -1..2 {
                        let key1 = (point[0] + x).to_string() + "," 
                            + &(point[1] + y).to_string() + ","
                            + &(point[2]+  z).to_string();
                        if let Some(indices1) = self.map.get(&key1) {
                            for i in indices0 {
                                for k in indices1 {
                                    func(*i, *k);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, point: &Vec3, i: usize) {
        if let Some(vec) = self.get_mut(point) {
            vec.push(i);
        }else{
            self.map.insert(self.get_spatial_key(point), vec![i]);
        }
    }

    pub fn get_mut(&mut self, point: &Vec3) -> Option<&mut Vec<usize>> {
        self.map.get_mut(&self.get_spatial_key(point))
    }

    pub fn get(&self, point: &Vec3) -> Vec<usize> {
        let base_x = (point.x / self.cell_size).floor() as i32;
        let base_y = (point.y / self.cell_size).floor() as i32;
        let base_z = (point.z / self.cell_size).floor() as i32;
        let mut indices = vec![];
        for x in -1..2 {
            for y in -1..2 {
                for z in -1..2 {
                    let key =  (base_x + x).to_string() + ","
                            + &(base_y + y).to_string() + ","
                            + &(base_z + z).to_string();
                    if let Some(idc) = self.map.get(&key) { // self.get_spatial_key(&(*point + vec3(x as f32, y as f32, z as f32) * self.cell_size)) 
                        indices.extend(idc);
                    }
                }
            }
        }
        //indices.dedup();
        indices //self.map.get(&self.get_spatial_key(point))
    }

    pub fn get_spatial_key(&self, point: &Vec3) -> String {
           (point.x / self.cell_size).floor().to_string() + "," 
        + &(point.y / self.cell_size).floor().to_string() + "," 
        + &(point.z / self.cell_size).floor().to_string() 
    }
}


// pub fn contains_key(&self, point: &Vec3) -> bool {
//     for x in -1..2 {
//         for y in -1..2 {
//             for z in -1..2 {
//                 if self.map.contains_key(&self.get_spatial_key(&(*point + vec3(x as f32, y as f32, z as f32)))) {
//                     return true;
//                 }
//             }
//         }
//     }
//     false
// }