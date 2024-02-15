use std::collections::HashMap;
use glam::*;

pub struct SpatialMap {
    pub map: HashMap<String, Vec<usize>>,
    cell_size: f32,
}

impl SpatialMap {
    pub fn new(cell_size: f32) -> SpatialMap {
        let map: HashMap<String, Vec<usize>> = HashMap::new();
        SpatialMap {
            map,
            cell_size,
        }
    }

    pub fn for_pairs<F>(&self, func: &mut F) where F: FnMut(usize, usize) {
        let mut key_parts = [0; 2];
        for (key0, indices0) in &self.map {
            for (i, string_int) in key0.split(",").enumerate() {
                key_parts[i] = string_int.parse().expect("failed to parse key in spatial");
            }
            for x in -1..2 {
                for y in -1..2 {
                    let key1 = (key_parts[0] + x).to_string() + ","
                        + &(key_parts[1]+y).to_string();
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

    pub fn insert(&mut self, point: &Vec2, i: usize) {
        if let Some(vec) = self.get_mut(point) {
            vec.push(i);
        }else{
            self.map.insert(self.get_spatial_key(point), vec![i]);
        }
    }

    pub fn get_mut(&mut self, point: &Vec2) -> Option<&mut Vec<usize>> {
        self.map.get_mut(&self.get_spatial_key(point))
    }

    pub fn get_spatial_key(&self, point: &Vec2) -> String {
        (point.x / self.cell_size).floor().to_string() + "," 
        + &(point.y / self.cell_size).floor().to_string() 
    }
}