//use super::vector::*;

pub struct Nurbs<T: IntoIterator<Item=f32>>  { // Iterator<Item=f32>  // where T: IntoIterator<Item=f32>
    pub order:   u8,        // order = polynomial_degree + 1  
    pub knots:   Vec<f32>,  // knot_count = order + vector_count 
    pub weights: Vec<f32>,  // weight_count = vector_count
    pub vectors: Vec<T>,    // vectors are control_points 
}

pub trait Discrete<T> {
    fn get_vector_at_u(&self, u: f32) -> Result<T, &'static str>;
}

impl<T: IntoIterator<Item=f32> + Default> Discrete<T> for Nurbs<T> {
    fn get_vector_at_u(&self, u: f32) -> Result<T, &'static str> {
        let vector = T::default();
        Ok(vector)
    }
}



// impl<T: IntoIterator<Item=f32> + Default> Nurbs<T> {
//     fn get_vector(self) -> Result<T, &'static str> {
//         let vector = T::default();
//         Ok(vector)
//     }
// }


// pub trait Discrete<T, const N: usize> {
//     fn get_vector(&self) -> Vec<Vector<T, N>>;
// }


// struct Nurbs<T> where T: IntoIterator<Item=f32> { // Iterator<Item=f32> 
//     order:   u8,        // order = polynomial_degree + 1  
//     knots:   Vec<f32>,  // knot_count = order + vector_count 
//     weights: Vec<f32>,  // weight_count = vector_count
//     vectors: Vec<T>,    // vectors are control_points 
// }

// pub trait Discrete<T> {
//     fn get_vector(&self) -> Vec<T>;
// }

// impl Discrete<T> for Nurbs<T> {
//     fn get_vector(&self) -> Vec<T> {
//         //self.vectors[0]
//     }
// }