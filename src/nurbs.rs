use crate::{mesh::{get_trivec, Mesh}, vector::get_transformed_vector};
use super::{Model, Parameter, DiscreteQuery, log};
use glam::*;
use serde::{Deserialize, Serialize};
use lyon::tessellation::*;
use rayon::prelude::*;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// #[derive(Clone, Default, Serialize, Deserialize)]
// #[serde(default = "Facet::default")]
// pub struct Facet {
//     pub surface:    Nurbs,
//     pub boundaries: Vec<Nurbs>,
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Nurbs::default")]
pub struct Nurbs {
    pub order:      usize,       // order = polynomial_degree + 1
    pub knots:      Vec<f32>,    // knot_count = order + control_count
    pub weights:    Vec<f32>,    // weight_count = control_count
    pub controls:   Vec<Model>,
    pub boundaries: Vec<Nurbs>,
}

impl Nurbs { // impl<T: Default + IntoIterator<Item=f32>> Nurbs<T> {
    pub fn get_transformed(&self, mat4: Mat4) -> Nurbs {
        let mut nurbs = Nurbs {
            order: self.order,
            knots: self.knots.clone(),
            weights: self.knots.clone(),
            controls: vec![],
            boundaries: self.boundaries.clone(),
        };
        for control in &self.controls {
            nurbs.controls.push(control.get_transformed(mat4));
        }
        nurbs
    }

    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh { // Vec<f32> {
        let nurbs = self.get_valid();
        let mut u_count = 0;
        for control in &nurbs.controls {
            if let Model::Curve(c) = control {
                //let potential_u_count = c.controls.len() + (c.controls.len()-1)*(c.order-2) * query.count;
                let sample_count = c.get_sample_count(query.count);
                if u_count < sample_count { u_count = sample_count; } 
            }
        }
        let v_count = nurbs.get_sample_count(query.count);
        //let v_count = nurbs.controls.len() + (nurbs.controls.len()-1)*(nurbs.order-2) * query.count;
        // let vector = (0..u_count).into_iter().map(|u|
        //     (0..v_count).into_iter()
        //         .map(|v| nurbs.get_vector_at_uv(u as f32 / (u_count-1) as f32,   v as f32 / (v_count-1) as f32))
        //         .collect::<Vec<Vec<f32>>>()
        //     ).flatten().flatten().collect();
        ///let max_count = *[u_count, v_count, query.count].iter().max().unwrap();

        let mut builder = lyon::path::Path::builder();
        //let curves = get_curves_from_parts(&self.parts);
        let mut started = false;
        let mut start_point = lyon::geom::Point::default();
        for curve in &nurbs.boundaries { // TODO: insert grid of tiny triangle boundaries to force extra points for curved surfaces
            for p in curve.get_polyline(query.count).chunks(3) {  
                let point = lyon::geom::Point::new(p[0], p[1]);
                if started {
                    if start_point.distance_to(point) > f32::EPSILON {
                        builder.line_to(point);
                    }else {
                        builder.end(true);
                        builder.begin(point);
                        start_point = point;
                    }
                }else{
                    builder.begin(point);
                    start_point = point;
                    started = true;
                }
            }
        }
        builder.end(true);
        let path = builder.build();
        let options = FillOptions::tolerance(query.tolerance/2.).with_intersections(true);
        let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
        let mut buffer_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
            vertex.position().to_array()
        });
        let mut tessellator = FillTessellator::new();
        tessellator.tessellate_path(&path, &options, &mut buffer_builder).unwrap(); //.expect("Tessellation failed");

        let mut vector = vec![];
        for [u, v] in geometry.vertices.into_iter(){
            vector.extend(nurbs.get_vector_at_uv(u, v));
        }

        Mesh {
            vector, //:    geometry.vertices.into_iter().flatten().collect(),
            triangles: geometry.indices.into_iter().map(|v| v as usize).collect(),
        }
    }

    pub fn get_polyline(&self, count: usize) -> Vec<f32> {
        let nurbs = self.get_valid();
        nurbs.get_polyline_at_t(&Parameter::U(0.), count)
    }

    pub fn get_polyline_at_t(&self, t: &Parameter, count: usize) -> Vec<f32> {
        let nurbs = self.get_valid();
        match t {
            Parameter::U(u) => nurbs.get_polyline_at_u(*u, count),
            Parameter::V(v) => nurbs.get_polyline_at_v(*v, count),
        }
    }

    pub fn get_controls_as_vec2(&self) -> Vec<Vec2> {
        self.controls.iter().map(|control| {
            if let Model::Point(p) = control {
                Vec2::new(p[0], p[1])
            }else{
                Vec2::default()
            }
        }).collect()
    }

    fn get_sample_count(&self, count: usize) -> usize { // sample_pulls: Vec<usize>
        let mul = self.controls.len()-1;
        self.controls.len() + count * (self.order - 2) * mul
    }

    fn get_polyline_at_u(&self, u: f32, count: usize) -> Vec<f32> {
        //let v_count = self.controls.len() + (self.controls.len()-1)*(self.order-2)*query.quality + ;
        let v_count = self.get_sample_count(count);
        (0..v_count).into_iter()
            .map(|t| self.get_vector_at_uv(u, t as f32 / (v_count-1) as f32)) 
            .flatten().collect()
    }

    fn get_polyline_at_v(&self, v: f32, count: usize) -> Vec<f32> {
        (0..count).into_iter()
            .map(|t| self.get_vector_at_uv(t as f32 / (count-1) as f32, v)) 
            .flatten().collect()
    }

    pub fn get_vector_at_uv(&self, u: f32, v: f32) -> Vec<f32> {
        let basis = self.get_rational_basis_at_t(v);
        let mut vector = vec![];
        for component_index in 0..self.get_control_vector(0, 0.).len() { 
            vector.push(
                (0..self.controls.len())
                    .map(|i| self.get_control_vector(i, u)[component_index] * basis[i]).sum()
            );
        }
        vector
    }

    fn get_control_vector(&self, index: usize, t: f32) -> Vec<f32> {
        match &self.controls[index] { 
            Model::Point(m) => m.to_vec(),  
            Model::Curve(m) => m.get_vector_at_uv(0., t),
            _ => vec![0.; 3], 
        }
    }

    fn get_rational_basis_at_t(&self, t: f32) -> Vec<f32> {
        let basis = self.get_basis_at_t(t);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    fn get_basis_at_t(&self, normal_t: f32) -> Vec<f32> {
        let t = *self.knots.last().unwrap_or(&0.) * normal_t; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_t(t);
        for degree in 1..self.order {
            for i0 in 0..self.controls.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] != 0. {
                    f = (t - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
                }
                if basis[i1] != 0. {
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

    fn get_basis_of_degree_0_at_t(&self, t: f32) -> Vec<f32> {
        self.knots.windows(2)
            .map(|knots| {
                if t >= knots[0] && t < knots[1] {
                    1.
                } else {
                    0.
                }
            }).collect()
    }

    fn get_open_knots(&self) -> Vec<f32> {
        let order = self.get_valid_order();
        let repeats = order - 1; // knot multiplicity = order for ends of knot vector
        let max_knot = self.controls.len() + order - (repeats * 2) - 1;
        let mut knots = vec![0_f32; repeats];
        knots.extend((0..=max_knot).map(|k| k as f32));
        knots.extend(vec![max_knot as f32; repeats]);
        knots
    }

    fn get_valid(&self) -> Nurbs {
        let mut controls = vec![];
        let mut make_bounds = false;
        for control in &self.controls {
            controls.push(self.get_valid_control(control));
            if let Model::Curve(_) = control {
                make_bounds = true;
            }
        }
        let mut boundaries = vec![];
        if make_bounds && self.boundaries.len() == 0 {
            // let mut curve0 = Nurbs::default();
            // let mut curve1 = Nurbs::default();
            // let mut curve2 = Nurbs::default();
            // //let mut curve1 = Nurbs::default();
            // curve0.controls.extend([Model::Point([0.1, 0.1, 0.]), Model::Point([0.9, 0.1, 0.])]);
            // curve1.controls.extend([Model::Point([0.9, 0.1, 0.]), Model::Point([0.9, 0.9, 0.])]);
            // curve2.controls.extend([Model::Point([0.9, 0.9, 0.]), Model::Point([0.1, 0.1, 0.])]);
            // boundaries.push(curve0.get_valid());
            // boundaries.push(curve1.get_valid());
            // boundaries.push(curve2.get_valid());
            let mut curve0 = Nurbs::default();
            //let mut curve1 = Nurbs::default();
            curve0.controls.extend([
                Model::Point([0., 0., 0.]), Model::Point([1., 0., 0.]),
                Model::Point([1., 1., 0.]), Model::Point([0., 1., 0.]), 
                Model::Point([0., 0., 0.]),
            ]);
            boundaries.push(curve0.get_valid());
            //console_log!("made bounds!!!!!")
        } else {
            boundaries = self.boundaries.clone();
        }
        Nurbs {
            order: self.get_valid_order(),
            knots: self.get_valid_knots(),
            weights: self.get_valid_weights(),
            controls, // : self.controls.iter().map(|c| self.get_valid_control(c.clone())).collect(), // self.controls.clone(), //
            boundaries,
        }
    }
    
    fn get_valid_control(&self, control: &Model) -> Model {
        match control {
            Model::Point(m) => Model::Point(*m),
            Model::Curve(m) => Model::Curve(m.get_valid()),
            _ => Model::Point([0.; 3]),
        }
    }

    fn get_valid_order(&self) -> usize {
        self.order.clamp(2, self.controls.len()) //if self.order == 0 {3} else {self.order.clamp(2, 10)}
    }

    fn get_valid_weights(&self) -> Vec<f32> {
        if self.weights.len() == self.controls.len() {
            self.weights.clone()
        } else {
            vec![1.; self.controls.len()]
        }
    }

    fn get_valid_knots(&self) -> Vec<f32> {
        if self.knots.len() == self.controls.len() + self.get_valid_order() {
            self.knots.clone()
        } else {
            self.get_open_knots()
        }
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



// let mut curve0 = Nurbs::default();
// //let mut curve1 = Nurbs::default();
// curve0.controls.extend([
//     Model::Point([0.1, 0.1, 0.]), Model::Point([0.9, 0.1, 0.]),
//     Model::Point([0.9, 0.9, 0.]), Model::Point([0.1, 0.9, 0.]), 
//     Model::Point([0.1, 0.1, 0.]),
// ]);
// boundaries.push(curve0.get_valid());
// console_log!("made bounds!!!!!")


// //let &DiscreteQuery {u_count, v_count, ..} = query;
// let quality_factor = 4;
// let nurbs = self.get_valid();
// let mut u_count = 0;
// for control in &nurbs.controls {
//     if let Model::Curve(c) = control {
//         let potential_u_count = c.controls.len() + (c.controls.len()-1)*(c.order-2)*quality_factor;
//         if u_count < potential_u_count { u_count = potential_u_count; } 
//     }
// }
// let v_count = nurbs.controls.len() + (nurbs.controls.len()-1)*(nurbs.order-2)*quality_factor;
// // let vector = (0..u_count).into_iter().map(|u|
// //     (0..v_count).into_iter()
// //         .map(|v| nurbs.get_vector_at_uv(u as f32 / (u_count-1) as f32,   v as f32 / (v_count-1) as f32))
// //         .collect::<Vec<Vec<f32>>>()
// //     ).flatten().flatten().collect();


        // match &self.controls[index] {
        //     Model::Vector(vector) =>   vector.to_vec(),
        //     Model::Nurbs(nurbs) =>     nurbs.get_vector_at_uv(u, 0.),
        //     //Model::Turtled(turtled) => turtled.get_vector_at_t(u),
        //     _ => self.controls[index].get_vector_at_t(u) // vec![0.; 3],
        // }



        // fn get_basis_at_t(&self, normal_t: f32) -> Vec<f32> {
//     let t = *self.knots.last().unwrap_or(&0.) * normal_t; // .unwrap_throw("") to js client
//     let mut basis = self.get_basis_of_degree_0_at_t(t);
//     for degree in 1..self.order {
//         for i0 in 0..self.controls.len() {
//             let i1 = i0 + 1; 
//             let mut f = 0.;
//             let mut g = 0.;
//             if basis[i0] > 0. {
//                 f = (t - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
//             }
//             if basis[i1] > 0. {
//                 g = (self.knots[degree + i1] - t) / (self.knots[degree + i1] - self.knots[i1])
//             }
//             basis[i0] = f * basis[i0] + g * basis[i1];
//         }
//     }
//     if normal_t == 1. {
//         basis[self.controls.len() - 1] = 1.; // last control edge case
//     }
//     basis
// }



        // fn get_basis_of_degree_0_at_t(&self, t: f32) -> Vec<f32> {
        //     let mut vector = vec![];
        //     for i in 0..self.knots.len()-1 { // 0..self.controls.len()-1 { //
        //         if t >= self.knots[i] && t < self.knots[i+1] { // self.order-1 + 
        //             vector.push(1.);
        //         //} else if i == self.knots.len()-2 && t >= self.knots[i+1] {
        //         //    vector.push(1.);
        //         } else {
        //             vector.push(0.);
        //         }
        //     }
        //     // if t > 0.99 {
        //     //     vector.extend([1.; 5]);
        //     // }else {
        //     //     vector.push(0.);
        //     // }
        //     vector
        // }


                // let mut vector = vec![];
        // for u in 0..max_control_count_u {
        //     for v in 0..v_count {
        //         //console_log!("v: {}", v);
        //         vector.extend(nurbs.get_vector_at_uv(v as f32 / (v_count-1) as f32,   u as f32 / (max_control_count_u-1) as f32));
        //     }
        // }
        // vector