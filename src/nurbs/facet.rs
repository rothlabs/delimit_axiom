use std::ops::DivAssign;

use crate::nurbs::Nurbs;
use crate::query::DiscreteQuery;
use crate::result::Mesh;
use crate::{Model, Shape, CurveShape, get_curves};
use glam::*;
use serde::{Deserialize, Serialize};
use lyon::tessellation::*;
use lyon::geom::{Box2D, Point};
use lyon::path::Winding;

//use rayon::prelude::*;

// ((a % b) + b) % b)  ->  a modulo b

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Facet::default")]
pub struct Facet {
    pub controls:    Vec<Model>,
    pub boundaries:  Vec<Model>,
    pub order:       usize,       // order = polynomial_degree + 1
    pub knots:       Vec<f32>,    // knot_count = order + control_count
    pub weights:     Vec<f32>,    // weight_count = control_count
    pub reversed:    bool,
}

impl Facet {
    pub fn get_shapes(&self) -> Vec<Shape> {
        vec![Shape::Facet(FacetShape{
            nurbs: Nurbs {
                order:   self.order,
                knots:   self.knots.clone(),
                weights: self.weights.clone(),
            },
            controls:   get_curves(&self.controls),
            boundaries: get_curves(&self.boundaries),
            reversed:   self.reversed,
        })]
    }
}

#[derive(Clone)]
pub struct FacetShape {
    pub nurbs:      Nurbs,
    pub controls:   Vec<CurveShape>,
    pub boundaries: Vec<CurveShape>,
    pub reversed:   bool,
}

impl Default for FacetShape {
    fn default() -> Self {
        FacetShape {
            nurbs: Nurbs::default(),
            controls: vec![],
            boundaries: vec![],
            reversed: false,
        }
    }
}

impl FacetShape { 
    pub fn get_reversed_and_transformed(&self, mat4: Mat4) -> Self {
        let mut facet = self.clone_with_empty_controls(true);
        for control in self.controls.iter().rev() {
            facet.controls.push(control.get_transformed(mat4));
        }
        facet
    }

    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        let mut facet = self.clone_with_empty_controls(false);
        for control in &self.controls {
            facet.controls.push(control.get_transformed(mat4));
        }
        facet
    }

    fn clone_with_empty_controls(&self, reversed: bool) -> Self {
        FacetShape {
            nurbs: self.nurbs.clone(),
            controls: vec![],
            boundaries: self.boundaries.clone(),
            reversed,
        }
    }

    pub fn get_param_step_and_samples(&self, min_count: usize, max_distance: f32) -> (Vec2, Vec<Vec2>) {
        let mut params = vec![];
        let mut u_count = 0;
        let mut average_v_controls = vec![];
        for curve in &self.controls {
            let sample_count = self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &curve.controls);
            if u_count < sample_count { u_count = sample_count; } 
            let mut point = Vec3::ZERO;
            for p in &curve.controls {
                point += *p;
            }
            average_v_controls.push(point / curve.controls.len() as f32);
        }
        let v_count = self.nurbs.get_sample_count_with_max_distance(min_count, max_distance, &average_v_controls);
        for u in 0..u_count {
            for v in 0..v_count {
                params.push(vec2(u as f32 / (u_count-1) as f32, v as f32 / (v_count-1) as f32));
            }
        }
        (vec2(1./(u_count-1) as f32, 1./(v_count-1) as f32), params)
    }

    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh { 
        let facet = self.get_valid();
        let mut u_count = 0;
        for curve in &facet.controls {
            let sample_count = curve.nurbs.get_sample_count(query.count);
            if u_count < sample_count { u_count = sample_count; } 
        }
        let v_count = facet.nurbs.get_sample_count(query.count);
        let mut builder = lyon::path::Path::builder();
        if facet.boundaries.is_empty() {
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
        for boundary in &facet.boundaries { 
            for p in boundary.get_polyline(query).chunks(3) {
                let mut y = p[1];
                if facet.reversed {y = 1.-y;}
                let point = lyon::geom::Point::new(p[0], y);
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
            vector.extend(facet.get_vector_at_uv(u, v));
        }
        Mesh {
            vector, //:    geometry.vertices.into_iter().flatten().collect(),
            trivec: geometry.indices, 
        }
    }

    pub fn get_vector_at_uv(&self, u: f32, v: f32) -> Vec<f32> {
        let basis = self.nurbs.get_rational_basis_at_u(v);
        let mut vector = vec![];
        if ! self.controls.is_empty() {
            for component_index in 0..3 { 
                vector.push(
                    (0..self.controls.len())
                        .map(|i| self.controls[i].get_vector_at_u(u)[component_index] * basis[i]).sum()
                );
            }
        }
        vector
    }

    fn get_valid(&self) -> FacetShape {
        FacetShape {
            nurbs: self.nurbs.get_valid(self.controls.len()),
            controls: self.controls.iter().map(|c| c.get_valid()).collect(), // self.controls.clone(), //
            boundaries: self.boundaries.clone(),
            reversed: self.reversed,
        }
    }
}


#[derive(Clone, Serialize, Deserialize)] 
pub enum Parameter {
    U(f32),
    V(f32),
}

impl Default for Parameter {
    fn default() -> Self { Parameter::U(0.) }
}


// // visual tests
// impl FacetShape {
//     // for examining the "basis functions" as pictured on wikipedia
//     pub fn get_basis_plot_vectors(&self, control_index: usize, count: usize) -> Vec<Vec<f32>> {
//         let max_t = *self.knots.last().unwrap_or(&0.); // .unwrap_throw("") to javascript client
//         (0..count)
//             .map(|t| {
//                 let x = (max_t / (count - 1) as f32) * t as f32;
//                 vec![x, self.get_basis_at_t(x)[control_index], 0.]
//             })
//             .collect()
//     }
// }