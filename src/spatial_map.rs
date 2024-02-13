use std::collections::HashMap;
use glam::*;

pub struct SpatialMap<T> {
    pub map: HashMap<String, T>,
    cell_size: f32,
}

impl<T: Clone> SpatialMap<T> {
    pub fn new(cell_size: f32) -> SpatialMap<T> {
        let map: HashMap<String, T> = HashMap::new();
        SpatialMap {
            map,
            cell_size,
        }
    }

    pub fn insert(&mut self, point: &Vec2, meta: &String, item: T) {
        //let s = self.cell_size;
        //for x in [point.x-s, point.x, point.x+s] {
        //    for y in [point.y-s, point.y, point.y+s] {
                self.map.insert(self.get_spatial_key(point.x, point.y, meta), item.clone());
        //    }
        //}
    }

    pub fn get_mut(&mut self, point: &Vec2, meta: &String) -> Option<&mut T> {
        // let s = self.cell_size;
        // for x in [point.x-s, point.x, point.x+s] {
        //     for y in [point.y-s, point.y, point.y+s] {
        //         self.map.get_mut(&self.get_spatial_key(point.x, point.y, meta))
        //     }
        // }
        self.map.get_mut(&self.get_spatial_key(point.x, point.y, meta))
    }

    pub fn contains_key(&self, point: &Vec2, meta: &String) -> bool { 
        self.map.contains_key(&self.get_spatial_key(point.x, point.y, meta))
    }
    // pub fn contains_key(&self, x: f32, y: f32, meta: &String) -> bool { //point: &Vec2
    //     self.map.contains_key(&self.get_spatial_key(x, y, meta))
    // }

    pub fn get_spatial_key(&self, x: f32, y: f32, meta: &String) -> String {
        (x/self.cell_size).round().to_string() + "," + &(y/self.cell_size).round().to_string() + "," + meta
    }
}

