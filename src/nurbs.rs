//use std::iter::zip;

//use super::vector::*;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use super::Model;
//use wasm_bindgen::UnwrapThrowExt;

// #[derive(Clone, Serialize, Deserialize)] //#[serde(default = "Control::default")]
// pub enum Control {
//     Vector(Vec<f32>),
//     Nurbs(Nurbs),
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Nurbs::default")]
pub struct Nurbs {
    // pub struct Nurbs<T: Default>  {
    pub order: usize,      // order = polynomial_degree + 1
    pub knots: Vec<f32>,   // knot_count = order + control_count
    pub weights: Vec<f32>, // weight_count = control_count
    pub controls: Vec<Model>,
}

impl Nurbs { // impl<T: Default + IntoIterator<Item=f32>> Nurbs<T> {
    pub fn get_valid_control(&self, control: Model) -> Model {
        match control {
            Model::Vector(control) => Model::Vector(control),
            Model::Nurbs(control) => Model::Nurbs(control.get_valid()),
            _ => Model::Vector(vec![0.; 3]),
        }
    }
    pub fn get_valid(&self) -> Nurbs {
        Nurbs {
            order: self.get_order(),
            knots: self.get_knots(),
            weights: self.get_weights(),
            controls: self.controls.iter().map(|c| self.get_valid_control(c.clone())).collect(),
        }
    }

    pub fn get_order(&self) -> usize {
        self.order.clamp(2, self.controls.len()) //if self.order == 0 {3} else {self.order.clamp(2, 10)}
    }

    pub fn get_weights(&self) -> Vec<f32> {
        if self.weights.len() == self.controls.len() {
            self.weights.clone()
        } else {
            vec![1.; self.controls.len()]
        }
    }

    pub fn get_knots(&self) -> Vec<f32> {
        if self.knots.len() == self.controls.len() + self.get_order() {
            self.knots.clone()
        } else {
            self.get_open_knots()
        }
    }

    pub fn get_open_knots(&self) -> Vec<f32> {
        let order = self.get_order();
        let repeats = order - 1; // knot multiplicity = order for ends of knot vector
        let max_knot = self.controls.len() + order - (repeats * 2) - 1;
        let mut knots = vec![0_f32; repeats];
        knots.extend((0..=max_knot).map(|k| k as f32));
        knots.extend(vec![max_knot as f32; repeats]);
        knots
    }

    pub fn get_basis_of_degree_0_at_t(&self, t: f32) -> Vec<f32> {
        self.knots.windows(2)
            .map(|knots| {
                if t >= knots[0] && t < knots[1] {
                    1.
                } else {
                    0.
                }
            })
            .collect()
    }

    pub fn get_basis_at_t(&self, normal_t: f32) -> Vec<f32> {
        let t = *self.knots.last().unwrap_or(&0.) * normal_t; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_t(t);
        for degree in 1..self.order {
            for i0 in 0..self.controls.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] > 0. {
                    f = (t - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
                }
                if basis[i1] > 0. {
                    g = (self.knots[degree + i1] - t) / (self.knots[degree + i1] - self.knots[i1])
                }
                basis[i0] = f * basis[i0] + g * basis[i1];
            }
        }
        if normal_t == 1. {
            basis[self.controls.len() - 1] = 1.; // last control edge case
        }
        basis
    }

    pub fn get_rational_basis_at_t(&self, t: f32) -> Vec<f32> {
        let basis = self.get_basis_at_t(t);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    pub fn get_control_vector(&self, index: usize, u: f32) -> Vec<f32> {
        match self.controls[index].clone() {
            Model::Vector(vector) => vector,
            Model::Nurbs(nurbs) => nurbs.get_vector_at_uv(u, 0.),
            _ => vec![0.; 3],
        }
    }

    pub fn get_vector_at_uv(&self, u: f32, v: f32) -> Vec<f32> {
        let basis = self.get_rational_basis_at_t(u);
        let mut vector = vec![];
        for component_index in 0..self.get_control_vector(0, 0.).len() { // 0..self.controls[0].len() {
            vector.push(
                (0..self.controls.len())
                    .map(|i| self.get_control_vector(i, v)[component_index] * basis[i]).sum()
            );
        }
        vector
    }

    pub fn get_vectors_at_u(&self, u: f32, count: usize) -> Vec<Vec<f32>> {
        (0..count).into_par_iter()
            .map(|t| self.get_vector_at_uv(u, t as f32 / (count-1) as f32)) // (max_t / (count - 1) as f32) * t as f32)
            .collect()
    }

    pub fn get_vectors_at_v(&self, v: f32, count: usize) -> Vec<Vec<f32>> {
        (0..count).into_par_iter()
            .map(|t| self.get_vector_at_uv(t as f32 / (count-1) as f32, v)) // (max_t / (count - 1) as f32) * t as f32)
            .collect()
    }

    pub fn get_surface_vectors(&self, u_count: usize, v_count: usize) -> Vec<Vec<f32>> {
        (0..u_count).into_par_iter().map(|u|
            (0..v_count).into_par_iter()
                .map(|v| self.get_vector_at_uv(v as f32 / (v_count-1) as f32, u as f32 / (u_count-1) as f32))
                .collect::<Vec<Vec<f32>>>()
            ).flatten().collect()
    }
}

// visual tests
impl Nurbs {
    // for examining the "basis functions" as pictured on wikipedia
    pub fn get_basis_plot_vectors(&self, control_index: usize, count: usize) -> Vec<Vec<f32>> {
        let max_t = *self.knots.last().unwrap_or(&0.); // .unwrap_throw("") to javascript client
        (0..count)
            .map(|t| {
                let x = (max_t / (count - 1) as f32) * t as f32;
                vec![x, self.get_basis_at_t(x)[control_index], 0.]
            })
            .collect()
    }
}






// let mut component = 0_f32;
// for i in 0..self.controls.len() {
//     component += basis[i] * self.controls[i][component_index];
// };
// vector.push(component);

// self.get_knots().skip_while(|k| ).windows(2).map(|knots| { // TODO: iterate over controls instead
//     if u >= knots[0] && u < knots[1] {1_f32} else {0_f32}
// }).collect()

// basis = zip(knots, basis).map(|x|{
//     let f = u
// }).collect();

// let knots = (0..self.controls.len()-1).map(|x| x as f32).collect::<Vec<f32>>();

// impl<T: Default + IntoIterator<Item=f32>> Nurbs<T> {
//     // fn get_basis_at_t(&self) -> Result<f32, &'static str> {

//     // }
// }

// fn get_vectors<T: IntoIterator<Item=f32> + Default>(nurbs: &Nurbs<T>) -> Vec<T> {

// }

// fn get_valid_nurbs<T: IntoIterator<Item=f32> + Default>(nurbs: &Nurbs<T>) -> Nurbs<T> {
//     let order = nurbs.order.clamp(2, 5);
//     Nurbs {
//         controls: nurbs.controls,
//         order,
//         knots: if nurbs.knots.len() == nurbs.controls.len() + order {nurbs.knots} else {get_open_knots(nurbs)}
//     }
// }

// struct Knots {
//     pub order: u8,
//     pub knots: ,
// }

//fn get_open_knots(nurbs: &NurbsQuery) ->

// pub trait Discrete<T> {
//     fn get_vector_at_t(&self, t: f32) -> Result<T, &'static str>;
// }

// impl<T: IntoIterator<Item=f32> + Default> Discrete<T> for Nurbs<T> {
//     fn get_vector_at_t(&self, t: f32) -> Result<T, &'static str> {
//         let vector = T::default();
//         Ok(vector)
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub struct DiscreteNurbs<T: IntoIterator<Item=f32>> {
//     pub nurbs: Nurbs<T>,
//     pub count: u32,
// }

// impl<T: IntoIterator<Item=f32>> DiscreteNurbs<T> {
//     pub fn get_vectors(&self) -> Result<Vec<f32>, &'static str> {
//         todo!()
//     }
// }

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
