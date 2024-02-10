use crate::mesh::Mesh;
use super::{Shape, Parameter, DiscreteQuery, log};
use glam::*;
use serde::{Deserialize, Serialize};
use lyon::tessellation::*;
use lyon::geom::{Box2D, Point};
use lyon::path::Winding;
//use rayon::prelude::*;

// ((a % b) + b) % b)  ->  a modulo b

// macro_rules! console_log {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Nurbs::default")]
pub struct Nurbs {
    pub order:      usize,       // order = polynomial_degree + 1
    pub knots:      Vec<f32>,    // knot_count = order + control_count
    pub weights:    Vec<f32>,    // weight_count = control_count
    pub controls:   Vec<Shape>,
    pub boundaries: Vec<Nurbs>,
}

impl Nurbs { // impl<T: Default + IntoIterator<Item=f32>> Nurbs<T> {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut is_facet = false;
        for control in &self.controls {
            if let Shape::Curve(_) = control {
                is_facet = true;
            }
        }
        if is_facet {
            vec![Shape::Facet(self.clone())]
        } else {
            vec![Shape::Curve(self.clone())]
        }
    }

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

    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh { 
        let nurbs = self.get_valid();
        let mut u_count = 0;
        for control in &nurbs.controls {
            if let Shape::Curve(c) = control {
                let sample_count = c.get_sample_count(query.count);
                if u_count < sample_count { u_count = sample_count; } 
            }
        }
        
        let v_count = nurbs.get_sample_count(query.count);
        let mut builder = lyon::path::Path::builder();
        if nurbs.boundaries.is_empty() {
            builder.add_rectangle(&Box2D{min:Point::new(0., 0.), max:Point::new(1., 1.)}, Winding::Positive);
        }
        for ui in 0..u_count {
            let u = ui as f32 / (u_count-1) as f32;
            builder.add_rectangle(&Box2D{min:Point::new(u, 0.), max:Point::new(u, 1.)}, Winding::Positive);
        }
        for vi in 0..v_count {
            let v = vi as f32 / (v_count-1) as f32;
            builder.add_rectangle(&Box2D{min:Point::new(0., v), max:Point::new(1., v)}, Winding::Positive);
        }
        let mut open_loop = false;
        let mut start_point = lyon::geom::Point::default();
        for curve in &nurbs.boundaries { 
            for p in curve.get_polyline(query).chunks(3) {  
                let point = lyon::geom::Point::new(p[0], p[1]);
                if open_loop {
                    if start_point.distance_to(point) > f32::EPSILON { // f32::EPSILON*1000.
                        builder.line_to(point);
                    }else {
                        builder.end(true);
                        open_loop = false;
                    }
                }else{
                    builder.begin(point);
                    start_point = point;
                    open_loop = true;
                }
            }
        }
        let path = builder.build();
        let options = FillOptions::default(); //tolerance(query.tolerance);
        let mut geometry: VertexBuffers<[f32; 2], usize> = VertexBuffers::new();
        let mut buffer_builder = BuffersBuilder::new(&mut geometry, |vertex: FillVertex| vertex.position().to_array());
        let mut tessellator = FillTessellator::new();
        tessellator.tessellate_path(&path, &options, &mut buffer_builder).expect("Tessellation failed");
        let mut vector = vec![];
        for [u, v] in geometry.vertices.into_iter(){
            vector.extend(nurbs.get_vector_at_uv(u, v));
        }
        Mesh {
            vector, //:    geometry.vertices.into_iter().flatten().collect(),
            triangles: geometry.indices, 
        }
    }

    pub fn get_polyline(&self, query: &DiscreteQuery) -> Vec<f32> {
        let nurbs = self.get_valid();
        nurbs.get_polyline_at_t(&Parameter::U(0.), query.count)
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
            if let Shape::Point(p) = control {
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
        if ! self.controls.is_empty() {
            for component_index in 0..self.get_control_vector(0, 0.).len() { 
                vector.push(
                    (0..self.controls.len())
                        .map(|i| self.get_control_vector(i, u)[component_index] * basis[i]).sum()
                );
            }
        }
        vector
    }

    fn get_control_vector(&self, index: usize, t: f32) -> Vec<f32> {
        //if index < self.controls.len(){
            match &self.controls[index] { 
                Shape::Point(m) => m.to_vec(),  
                Shape::Curve(m) => m.get_vector_at_uv(0., t),
                _ => vec![0.; 3], 
            }
        //}else{
        //    vec![0.; 3]
        //}
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
        Nurbs {
            order: self.get_valid_order(),
            knots: self.get_valid_knots(),
            weights: self.get_valid_weights(),
            controls: self.controls.iter().map(|c| self.get_valid_control(c)).collect(), // self.controls.clone(), //
            boundaries: self.boundaries.clone(),
        }
    }
    
    fn get_valid_control(&self, control: &Shape) -> Shape {
        match control {
            Shape::Point(m) => Shape::Point(*m),
            Shape::Curve(m) => Shape::Curve(m.get_valid()),
            _ => Shape::Point([0.; 3]),
        }
    }

    fn get_valid_order(&self) -> usize {
        self.order.min(2).max(self.controls.len())
        //log("clamp");
        //let wow = self.order.clamp(2, self.controls.len()); //if self.order == 0 {3} else {self.order.clamp(2, 10)}
        //log("clamp worked");
        //wow
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