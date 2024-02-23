use serde::{Deserialize, Serialize};
use crate::Model;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "DiscreteQuery::default")]
pub struct DiscreteQuery {
    pub model:     Model,
    pub count:     usize,
    pub tolerance: f32,   
    //pub one_mesh:  bool,
}

impl DiscreteQuery {
    pub fn get_valid(self) -> DiscreteQuery {
        let mut count = 8;
        if self.count > 0 { 
            count = self.count.clamp(2, 100); 
        }
        let mut tolerance = 0.1;
        if self.tolerance > 0. { 
            tolerance = self.tolerance.clamp(0.01, 10.); 
        }
        DiscreteQuery {
            model: self.model,
            count,
            tolerance,
            //one_mesh: self.one_mesh,
        }
    }
}