use std::f32::{EPSILON, INFINITY};
use std::ops::DivAssign;

use crate::nurbs::Nurbs;
use crate::query::DiscreteQuery;
use crate::scene::Mesh;
use crate::{get_curves, get_line_intersection, hash_vector, log, CurveShape, Model, Shape};
use glam::*;
use lyon::path::path::BuilderImpl;
use serde::{Deserialize, Serialize};
use lyon::tessellation::*;
use lyon::geom::{Box2D, Point};
use lyon::path::Winding;
use lyon::path::builder::NoAttributes;



//use wasm_bindgen_test::console_log;

//use rayon::prelude::*;

// ((a % b) + b) % b)  ->  a modulo b

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Facet::default")]
pub struct Facet {
    pub controls:    Vec<Model>,
    pub boundaries:  Vec<Model>,
    pub order:       usize,       // order = polynomial_degree + 1
    pub knots:       Vec<f32>,    // knot_count = order + control_count
    pub weights:     Vec<f32>,    // weight_count = control_count
    //pub reversed:    bool,
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
            //reversed:   self.reversed,
            perimeter:  false,
        })]
    }
}

#[derive(Clone)]
pub struct FacetShape {
    pub nurbs:      Nurbs,
    pub controls:   Vec<CurveShape>,
    pub boundaries: Vec<CurveShape>,
    //pub reversed:   bool,
    pub perimeter: bool,
}

impl Default for FacetShape {
    fn default() -> Self {
        FacetShape {
            nurbs: Nurbs::default(),
            controls: vec![],
            boundaries: vec![],
            //reversed: false,
            perimeter: false,
        }
    }
}

impl FacetShape { 
    pub fn get_transformed(&self, mat4: Mat4) -> Self {
        let mut facet = self.clone_with_empty_controls_and_boundaries();
        for control in &self.controls {
            facet.controls.push(control.get_transformed(mat4));
        }
        facet.boundaries = self.boundaries.clone();
        facet
    }

    pub fn get_transformed_and_reversed(&self, mat4: Mat4) -> Self {
        let mut facet = self.clone_with_empty_controls_and_boundaries();
        facet.nurbs.weights.reverse();
        for control in self.controls.iter().rev() {
            facet.controls.push(control.get_transformed(mat4)); //  * Mat4::from_scale(vec3(0., 0., 0.))
        }
        for bndry in &self.boundaries {
            facet.boundaries.push(bndry.get_transformed(
                Mat4::from_translation(vec3(0., 1., 0.)) * Mat4::from_scale(vec3(1., -1., 1.))
            ));
        }
        facet
    }

    fn clone_with_empty_controls_and_boundaries(&self) -> Self {
        FacetShape {
            nurbs: self.nurbs.clone(),
            controls: vec![],
            boundaries: vec![],
            perimeter: self.perimeter,
        }
    }

    pub fn get_uv_and_point_from_3d_dir(&self, uv: Vec2, dir: Vec3) -> (Vec2, Vec3) {
        let mut step_u = 0.0001;
        let mut step_v = 0.0001;
        if uv.x + step_u > 1. {step_u = -step_u;}
        if uv.y + step_v > 1. {step_v = -step_v;}
        let p0 = self.get_point_at_uv(uv);
        let pu = self.get_point_at_uv(uv + Vec2::X * step_u);
        let pv = self.get_point_at_uv(uv + Vec2::Y * step_v);
        let length_ratio_u = dir.length() / p0.distance(pu) * step_u;
        let length_ratio_v = dir.length() / p0.distance(pv) * step_v;
        let uv_dir = vec2(
            (pu-p0).normalize().dot(dir.normalize()) * length_ratio_u, 
            (pv-p0).normalize().dot(dir.normalize()) * length_ratio_v
        );
        let mut uv1 = uv;
        if uv_dir.length() > 0.0001 {
            uv1 = uv + uv_dir; 
            if uv1.x > 1. {uv1 = get_line_intersection(uv, uv + uv_dir*1000., Vec2::X, Vec2::ONE).unwrap();}
            if uv1.x < 0. {uv1 = get_line_intersection(uv, uv + uv_dir*1000., Vec2::ZERO, Vec2::Y).unwrap();}
            if uv1.y > 1. {uv1 = get_line_intersection(uv, uv + uv_dir*1000., Vec2::Y, Vec2::ONE).unwrap();}
            if uv1.y < 0. {uv1 = get_line_intersection(uv, uv + uv_dir*1000., Vec2::ZERO, Vec2::X).unwrap();}
        }
        uv1 = uv1.clamp(Vec2::ZERO, Vec2::ONE); // TODO: might not be needed
        // if uv1.x > 1. || uv1.x < 0. || uv1.y < 0. || uv1.x > 1. {
        //     console_log!("over bounds! {},{}", uv.x, uv.y);
        // }
        let point = self.get_point_at_uv(uv1);
        (uv1, point)
    }

    pub fn get_normal_at_uv(&self, uv: Vec2) -> Vec3 {
        let mut step_u = 0.0001;
        let mut step_v = 0.0001;
        if uv.x + step_u > 1. {step_u = -step_u;}
        if uv.y + step_v > 1. {step_v = -step_v;}
        let p0 = self.get_point_at_uv(uv);
        let p1 = self.get_point_at_uv(uv + Vec2::X * step_u);
        let p2 = self.get_point_at_uv(uv + Vec2::Y * step_v);
        step_u.signum() * step_v.signum() * (p0 - p1).normalize().cross((p0 - p2).normalize()).normalize() // TODO: remove final normalize after Union3 works!!!!
    }

    pub fn get_point_at_uv(&self, uv: Vec2) -> Vec3 {
        let p = self.get_vector_at_uv(uv.x, uv.y);
        vec3(p[0], p[1], p[2])
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
            //let average_point = point / curve.controls.len() as f32 / 8.;
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
        if self.perimeter || facet.boundaries.is_empty() {
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
        let mut loop_open = false;
        let mut bndry_i = 0;
        let mut start_bndry_i = bndry_i;
        let mut used_boundaries = vec![];
        //let mut perim_factors = [0.1, 0.01, 0.01, 1.];
        for _ in 0..facet.boundaries.len() {
            let bndry = &facet.boundaries[bndry_i];
            for p in bndry.get_polyline_vector(query).chunks(3) {
                let mut y = p[1];
                //////if facet.reversed {y = 1.-y;}
                let point = lyon::geom::Point::new(p[0], y);
                if loop_open {
                    builder.line_to(point);
                }else{
                    builder.begin(point);
                    loop_open = true;
                }
            }
            bndry_i = facet.next_boundary(&bndry.get_point_at_u(1.), &mut used_boundaries, &mut builder, 0);
            used_boundaries.push(bndry_i);
            if bndry_i == start_bndry_i {
                builder.end(true);
                loop_open = false;
                for i in 0..facet.boundaries.len() {
                    if !used_boundaries.contains(&i) {
                        bndry_i = i;
                        start_bndry_i = i;
                        break;
                    }
                }
            }
        }
        builder.end(true);
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
            digest: hash_vector(&vector), 
            vector, //:    geometry.vertices.into_iter().flatten().collect(),
            trivec: geometry.indices, 
        }
    }

    fn next_boundary(&self, point: &Vec3, used_boundaries: &mut Vec<usize>, builder: &mut NoAttributes<BuilderImpl>, iteration: usize) -> usize {
        if iteration > 3 {return 0;}
        let mut bndry_i = 0;
        let mut distance = INFINITY;
        let mut boundaries_x0 = vec![];
        let mut boundaries_x1 = vec![];
        let mut boundaries_y0 = vec![];
        let mut boundaries_y1 = vec![];
        for (i, curve) in self.boundaries.iter().enumerate() { 
            let p1 = curve.get_point_at_u(0.);
            let dist = point.distance(p1);
            if distance > dist {
                distance = dist;
                bndry_i = i;
            }
            if !used_boundaries.contains(&i) {
                if p1.x <    EPSILON {boundaries_x0.push((i, p1)); } // left
                if p1.y <    EPSILON {boundaries_y0.push((i, p1)); } // bottom
                if p1.x > 1.-EPSILON {boundaries_x1.push((i, p1)); } // right
                if p1.y > 1.-EPSILON {boundaries_y1.push((i, p1)); } // top
            }
        }
        if point.x < EPSILON && point.y > EPSILON  { // left of boundbox  && point.y < 1.-EPSILON
            boundaries_x0.sort_by(|a, b| b.1.y.partial_cmp(&a.1.y).unwrap());
            if let Some(a) = boundaries_x0.iter().next(){ // .filter(|a| a.1.y < point.y)
                bndry_i = a.0;
                //used_boundaries.push(a.0);
            }else{
                builder.line_to(lyon::geom::Point::new(0., 0.));
                bndry_i = self.next_boundary(&vec3(0., 0., 0.), used_boundaries, builder, iteration+1);
            }
        } else if point.y < EPSILON && point.x < 1.-EPSILON { // bottom of boundbox  point.x > EPSILON && 
            boundaries_y0.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap());
            if let Some(a) = boundaries_y0.iter().next(){ // .filter(|a| a.1.x > point.x)
                bndry_i = a.0;
                //used_boundaries.push(a.0);
            }else{
                builder.line_to(lyon::geom::Point::new(1., 0.));
                bndry_i = self.next_boundary(&vec3(1., 0., 0.), used_boundaries, builder, iteration+1);
            }
        } else if point.x > 1.-EPSILON && point.y < 1.-EPSILON { // right of boundbox // point.y > EPSILON &&
            boundaries_x1.sort_by(|a, b| a.1.y.partial_cmp(&b.1.y).unwrap());
            if let Some(a) = boundaries_x1.iter().next(){ // .filter(|a| a.1.y > point.y)
                bndry_i = a.0;
                //used_boundaries.push(a.0);
            }else{
                builder.line_to(lyon::geom::Point::new(1., 1.));
                //used_boundaries.push(bi);
                bndry_i = self.next_boundary(&vec3(1., 1., 0.), used_boundaries, builder, iteration+1);
            }
        } else if point.y > 1.-EPSILON && point.x > EPSILON { // top of boundbox  // && point.x < 1.-EPSILON
            boundaries_y1.sort_by(|a, b| b.1.x.partial_cmp(&a.1.x).unwrap());
            if let Some(a) = boundaries_y1.iter().next(){ // .filter(|a| a.1.x < point.x)
                bndry_i = a.0;
                //used_boundaries.push(a.0);
            }else{
                builder.line_to(lyon::geom::Point::new(0., 1.));
                bndry_i = self.next_boundary(&vec3(0., 1., 0.), used_boundaries, builder, iteration+1);
            }
        }
        bndry_i
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

    pub fn get_valid(&self) -> FacetShape {
        FacetShape {
            nurbs: self.nurbs.get_valid(self.controls.len()),
            controls: self.controls.iter().map(|c| c.get_valid()).collect(), // self.controls.clone(), //
            boundaries: self.boundaries.clone(),
            //reversed: self.reversed,
            perimeter:  self.perimeter,
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



// pub fn get_uv_and_point_from_3d_dir(&self, uv: Vec2, dir: Vec3) -> (Vec2, Vec3) {
//     let step = 0.0001;
//     let p0 = self.get_point_at_uv(uv);
//     let pu = self.get_point_at_uv(uv + Vec2::X * step);
//     let pv = self.get_point_at_uv(uv + Vec2::Y * step);
//     let uv_dir = vec2((pu-p0).normalize().dot(dir.normalize()), (pv-p0).normalize().dot(dir.normalize()));
//     let pd = self.get_point_at_uv(uv + Vec2::ONE.normalize()*step);
//     let length_ratio = (dir.length() / p0.distance(pd)) * step;
//     let uv1 = (uv + uv_dir * length_ratio).clamp(Vec2::ZERO, Vec2::ONE); // TODO: should be limited by length instead of component wise!!
//     //console_log!("uv: {},{}", uv1.x, uv1.y);
//     let point = self.get_point_at_uv(uv1);
//     (uv1, point)
// }


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