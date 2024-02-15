use crate::{log};
use std::collections::HashMap;
use glam::*;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub struct SpatialMap<T> {
    pub map: HashMap<String, Vec<usize>>,
    data: Vec<T>,
    cell_size: f32,
}

impl<T: Clone> SpatialMap<T> {
    pub fn new(cell_size: f32) -> SpatialMap<T> {
        let map: HashMap<String, Vec<usize>> = HashMap::new();
        SpatialMap {
            map,
            data: vec![],
            cell_size,
        }
    }

    pub fn get_pairs(&self) -> Vec<[T; 2]> {
        let mut result = vec![];
        let mut key_parts = [0; 2];
        for (key0, indices0) in &self.map {
            for (i, string_int) in key0.split(",").enumerate() {
                key_parts[i] = string_int.parse().expect("failed to parse key in union");
            }
            for x in -1..2 {
                for y in -1..2 {
                    let key1 = (key_parts[0]+x).to_string()+","+&(key_parts[1]+y).to_string();// + ",";
                    if let Some(indices1) = self.map.get(&key1) {
                        for i in indices0 {
                            for k in indices1 {
                                result.push([self.data[*i].clone(), self.data[*k].clone()]);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn insert(&mut self, point: &Vec2, meta: &String, item: &T) {
        self.data.push(item.clone());
        let i = self.data.len() - 1;
        if let Some(vec) = self.get_mut(point.x, point.y, &meta) {
            vec.push(i);
        }else{
            self.map.insert(self.get_spatial_key(point.x, point.y, meta), vec![i]);
        }
    }

    pub fn get_mut(&mut self, x: f32, y: f32, meta: &String) -> Option<&mut Vec<usize>> {
        self.map.get_mut(&self.get_spatial_key(x, y, meta))
    }

    pub fn get_spatial_key(&self, x: f32, y: f32, meta: &String) -> String {
        (x/self.cell_size).floor().to_string() + "," 
        + &(y/self.cell_size).floor().to_string() + meta
    }
}

// pub fn contains_key(&self, point: &Vec2, meta: &String) -> bool { 
//     self.map.contains_key(&self.get_spatial_key(point.x, point.y, meta))
// }
// // pub fn contains_key(&self, x: f32, y: f32, meta: &String) -> bool { //point: &Vec2
// //     self.map.contains_key(&self.get_spatial_key(x, y, meta))
// // }

// pub fn get_mut(&mut self, point: &Vec2, meta: &String) -> Option<&mut T> {
//     // let s = self.cell_size;
//     // for x in [point.x-s, point.x, point.x+s] {
//     //     for y in [point.y-s, point.y, point.y+s] {
//     //         self.map.get_mut(&self.get_spatial_key(point.x, point.y, meta))
//     //     }
//     // }
//     self.map.get_mut(&self.get_spatial_key(point.x, point.y, meta))
// }


// pub fn get_pairs(&self) -> Vec<[T; 2]> {
//     let mut result = vec![];
//     for (_, indices) in &self.map {
//         for i in indices {
//             for k in indices {
//                 result.push([self.data[*i].clone(), self.data[*k].clone()]);
//             }
//         }
//     }
//     result
// }

// pub fn insert(&mut self, point: &Vec2, meta: &String, item: &T) {
//     self.data.push(item.clone());
//     let i = self.data.len() - 1;
//     let s = self.cell_size;
//     for x in [point.x-s, point.x, point.x+s] {
//         for y in [point.y-s, point.y, point.y+s] {
//             if let Some(vec) = self.get_mut(x, y, &meta) {
//                 vec.push(i);
//             }else{
//                 self.map.insert(self.get_spatial_key(x, y, meta), vec![i]);
//             }
//             //self.map.insert(self.get_spatial_key(x, y, meta), i);
//         }
//     }
// }