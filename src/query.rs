use serde::{Deserialize, Serialize};
use crate::Model;

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "DiscreteQuery::default")]
pub struct DiscreteQuery {
    pub model:     Model,
    pub count:     usize,
    pub tolerance: f32,   
}

impl DiscreteQuery {
    pub fn get_valid(self) -> DiscreteQuery {
        let mut count = 50; // 8
        if self.count > 0 { 
            count = self.count.clamp(50, 100); 
        }
        let mut tolerance = 0.1;
        if self.tolerance > 0. { 
            tolerance = self.tolerance.clamp(0.01, 10.); 
        }
        DiscreteQuery {
            model: self.model,
            count,
            tolerance,
        }
    }
}