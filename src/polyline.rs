
use super::Model;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
//use lyon::path:FromPolyline;

pub trait Polyline {
    fn get_polyline(&self, count: usize) -> Vec<f32>;
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "PolylineQuery::default")]
struct PolylineQuery { 
    model: Model,
    count: usize,
    tolerance: f32,
}

#[wasm_bindgen]
pub fn get_polyline(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: PolylineQuery = serde_wasm_bindgen::from_value(val)?; 
    let count = queried.count.clamp(2, 10000);
    let tolerance = queried.tolerance.clamp(0.1, 1.);
    let polyline = match queried.model {
        Model::Nurbs(nurbs)      =>    nurbs.get_polyline(count),
        Model::Slice(slice)      =>    slice.get_polyline(count),
        Model::Turtled(turtled)  =>  turtled.get_polyline(count),
        Model::Path2D(path_2d)   =>  path_2d.get_polyline(tolerance),
        _ => vec![0.; 6],
    };
    Ok(serde_wasm_bindgen::to_value(&polyline)?)
}


// pub struct PolylineIterator {
//     vector: Vec<f32>,
//     index: usize,
// }

// impl PolylineIterator {
//     pub fn new(vector: Vec<f32>) -> Self {
//         PolylineIterator {
//             vector,
//             index: 0,
//         }
//     }
// }

// impl Iterator for PolylineIterator {
//     type Item = PathEvent;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index + 4 < self.vector.len() {
//             let from = point(self.vector[self.index], self.vector[self.index+1]);
//             let to = point(self.vector[self.index+3], self.vector[self.index+4]);
//             self.index += 3;
//             Some(PathEvent::Line { from, to })
//         } else {
//             None
//         }
//     }
// }





//let vector = queried.model.get_polyline(count); //queried.get_polyline();//get_polyline_vector(queried.model, count);


// impl Polyline for Model {
//     fn get_polyline(&self, count: usize) -> Vec<f32> {
//         self.get_polyline(count)
//     }
// }

// impl <T> PolylineQuery <T:Polyline> {
//     fn get_polyline(&self) -> Vec<f32> {
//         let count = self.count.clamp(2, 10000);
//         match self.model {
//             Model::T(nurbs) => nurbs.get_polyline(count),
//             _ => vec![0.; 6],
//         }
//     }
// }


// fn get_polyline_vector(model: Model, count: usize) -> Vec<f32> {
//     match model {
//         Model::Nurbs(nurbs) => nurbs.get_polyline(count),
//         Model::Slice(slice) => slice.get_polyline(count),
//         _ => vec![0.; 6],
//     }
// }



// impl PolylineQuery {
//     fn get_polyline(&self) -> Vec<f32> {
//         let count = self.count.clamp(2, 10000);
//         //self.model.get_polyline(self.count)
//         match self.model {

//         }
//     }
// }

// Model::SliceAtU(slice) => slice.get_polyline_at_u(count),
        // Model::SliceAtV(slice) => slice.get_polyline_at_v(count),