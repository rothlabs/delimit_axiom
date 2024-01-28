
use super::Model;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub trait Polyline {
    fn get_polyline(&self, count: usize) -> Vec<f32>;
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "PolylineQuery::default")]
struct PolylineQuery { 
    model: Model,
    count: usize,
}

#[wasm_bindgen]
pub fn get_polyline(val: JsValue) -> Result<JsValue, JsValue> {
    let queried: PolylineQuery = serde_wasm_bindgen::from_value(val)?; 
    let count = queried.count.clamp(2, 10000);
    let polyline = match queried.model {
        Model::Nurbs(nurbs)     =>   nurbs.get_polyline(count),
        Model::Slice(slice)     =>   slice.get_polyline(count),
        Model::Turtled(turtled) => turtled.get_polyline(count),
        _ => vec![0.; 6],
    };
    Ok(serde_wasm_bindgen::to_value(&polyline)?)
}


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