use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self {x:tuple.0, y:tuple.1, z:tuple.2}
    }
}
// let my_vector = Vector3::from((4.7, 5.2, 8.9));

impl IntoIterator for Vector3 {
    type Item = f32;
    type IntoIter = std::array::IntoIter<f32, 3>;
    fn into_iter(self) -> Self::IntoIter { // 
        IntoIterator::into_iter([self.x, self.y, self.z]) // std::array::IntoIter::new
    }
}


// impl Default for Vector3 {
//     fn default() -> Vector3 {
//         Vector3 {x:16.7, y:43.8, z:59.1}
//     }
// }


// another approach:

// #[derive(Debug, Clone, Copy)]
// pub struct Vector<T, const N: usize> {
//     data: [T; N],
// }

// impl <T, const N: usize> From <[T; N]> for Vector<T, N> {
//     fn from(data: [T; N]) -> Self {
//         Self {data}
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     fn from_impl(){
//         let _vec: Vector<f32, 3> = [4.2, 1.3, 7.8].into();
//     }
// }






// impl IntoIterator for &Vector3 {
//     type Item = f32;
//     type IntoIter = Vector3Iterator;  // type IntoIter = ::std::vec::IntoIter<f32>;
//     fn into_iter(self) -> Self::IntoIter {
//         Vector3Iterator {
//             vector: *self,
//             component_index: 0,
//         }
//     }
// }

// struct Vector3Iterator {
//     vector: Vector3,
//     component_index: u8,
// }

// impl Iterator for Vector3Iterator {
//     type Item = f32;
//     fn next(&mut self) -> Option<f32> {
//         self.component_index += 1;
//         match self.component_index {
//             1 => Some(self.vector.x),
//             2 => Some(self.vector.y),
//             3 => Some(self.vector.z),
//             _ => {
//                 self.component_index = 0; 
//                 None
//             },
//         }
//     }
// }






// struct Vector5 {
//     x: f32,
//     y: f32,
//     z: f32,
//     a: f32,
//     b: f32,
//     component_index: u8,
// }

// impl Iterator for Vector5 {
//     type Item = f32;
//     fn next(&mut self) -> Option<f32> {
//         self.component_index += 1;
//         match self.component_index {
//             1 => Some(self.x),
//             2 => Some(self.y),
//             3 => Some(self.z),
//             4 => Some(self.a),
//             5 => Some(self.b),
//             _ => {
//                 self.component_index = 0; 
//                 None
//             },
//         }
//     }
// }