
use super::{Model, DiscreteQuery};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_polyline(val: JsValue) -> Result<JsValue, JsValue> {
    let query: DiscreteQuery = serde_wasm_bindgen::from_value(val)?; 
    let query = query.get_valid();
    let polyline = query.model.get_polyline(&query);
    Ok(serde_wasm_bindgen::to_value(&polyline)?)
}

impl Model {
    pub fn get_polyline(&self, query: &DiscreteQuery) -> Vec<f32> {
        let DiscreteQuery {count, tolerance, ..} = query;
        match self {
            Model::Nurbs(m)     => m.get_polyline(*count),
            Model::Slice(m)     => m.get_polyline(*count),
            Model::Turtled(m)   => m.get_polyline(*count),
            Model::Path(m)      => m.get_polyline(*tolerance),
            Model::Circle(m)    => m.get_polyline(*tolerance),
            Model::Rectangle(m) => m.get_polyline(*tolerance),
            _ => vec![],
        }
    }
    pub fn get_polylines(&self, query: &DiscreteQuery) -> Vec<Vec<f32>> {
        match &self {
            Model::Group(m) => m.get_polylines(query),
            _ => vec![self.get_polyline(query)],
        }
    }
}


// pub trait Polyline {
//     fn get_polyline(&self, count: usize) -> Vec<f32>;
// }


// #[derive(Default, Serialize, Deserialize)]
// #[serde(default = "PolylineQuery::default")]
// pub struct PolylineQuery { 
//     model: Model,
//     count: usize,
//     tolerance: f32,
// }

// let mut count = 80;
// if query.count > 0 { count = query.count.clamp(2, 10000); }
// let tolerance = query.tolerance.clamp(0.01, 10.);
// let query = DiscreteQuery {
//     tolerance,
//     count,
//     ..query
// };


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