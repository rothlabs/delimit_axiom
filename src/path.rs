use crate::{DiscreteQuery, Nurbs};

use super::{Model, log};
use lyon::geom::euclid::Point2D;
use lyon::math::{Box2D, Angle, Vector, vector, Point, point};
use lyon::path::{ArcFlags, Event, PathEvent, Winding}; // PathBuffer
use lyon::path::builder::BorderRadii;
use lyon::path::traits::{PathIterator, SvgPathBuilder};
use serde::{Deserialize, Serialize};
use glam::*;
//use wasm_bindgen_test::console_log;

impl Model {
    pub fn get_path(&self) -> lyon::path::Path {
        //let default = lyon::path::Path::default().clone();
        match self {
            Model::Path(m)      => m.get_path().clone(), 
            Model::Circle(m)    => m.get_path().clone(),
            Model::Rectangle(m) => m.get_path().clone(),
            _ => lyon::path::Path::default(),
        }
    }
    pub fn add_paths_to_builder(&self, builder: &mut lyon::path::path::Builder) { //  -> Vec<lyon::path::Path>
        match self {
            Model::Group(m) => {
                for path in m.get_paths(){
                    builder.extend_from_paths(&[path.as_slice()]);
                }
            } 
            _ => builder.extend_from_paths(&[self.get_path().as_slice()]), //vec![self.get_path()],//lyon::path::PathSlice::default()
        }
    }
    // pub fn get_vector_at_t(&self, t: f32) -> Vec<f32> {
    //     //log("model get_vector_at_t");
    //     match self {
    //         //Model::Path(m) => m.get_vector_at_t(t),
    //         Model::Path(m) => {m.get_vector_at_t(t)}, // log("Model::Path get_vector_at_t"); 
    //         Model::Group(m) => {m.get_vector_at_t(t)}, // log("Model::group get_vector_at_t"); 
    //         Model::Vector(vector) => {vector.to_vec()}, // log("Model::vector get_vector_at_t"); 
    //         Model::Nurbs(nurbs) =>   {nurbs.get_vector_at_uv(t, 0.)}, // log("Model::nurbs get_vector_at_t"); 
    //         _ => {vec![0.;3]}, // log("Model::empty get_vector_at_t"); 
    //     }
    // }
}

pub fn get_path_from_parts(parts: &Vec<Model>) -> lyon::path::Path {
    let mut builder = lyon::path::Path::builder();
    for part in parts {
        part.add_paths_to_builder(&mut builder);
    }
    builder.build()
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Path::default")]
pub struct Path {
    pub parts: Vec<Model>,
    pub reverse: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "ArcTo::default")]
pub struct ArcTo {
    pub to:    Box<Model>,
    pub radii: Box<Model>,
}

impl Path { 
    // pub fn get_vector_at_t(&self, t: f32) -> Vec<f32> {
    //     let path = self.get_path();
    //     get_vector_at_t(&path, t)
    //     // let [x, y] = get_vector_at_t(&path, t).to_array();
    //     // vec![x, y, 0.]
    // }
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        get_polyline(self.get_path().clone(), tolerance)
    }
    pub fn get_path(&self) -> lyon::path::Path { 
        let mut builder = lyon::path::Path::builder();
        for part in &self.parts {
            part.add_paths_to_builder(&mut builder);
        }
        let mut svg_builder = builder.with_svg();//let mut svg_builder = lyon::path::Path::svg_builder();
        for part in &self.parts {
            match part {
                Model::MoveTo(p) => {svg_builder.move_to(get_point(p)); ()}, // builder.begin
                Model::LineTo(p) => {svg_builder.line_to(get_point(p)); ()},
                Model::ArcTo(p)  => {
                    svg_builder.arc_to(get_vector(&*p.radii), Angle::default(), ArcFlags::default(), get_point(&*p.to)); 
                    ()
                },
                Model::Close(_)  => svg_builder.close(),
                _ => (),
            };
        }
        if self.reverse { 
            build_path_from_iterator(svg_builder.build().reversed())
        }else{
            svg_builder.build()
        }
    }
}

// macro_rules! console_log {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }
pub fn get_nurbs_from_path(path: &lyon::path::Path) -> Nurbs { // , query: &DiscreteQuery
    let mut iterator = path.into_iter().flattened(0.1);
    let mut nurbs = Nurbs::default();
    let mut last_point = Point2D::default(); 
    while let Some(segment) = iterator.next() {
        let start = segment.from();
        nurbs.controls.push(Model::Vector(vec![start.x, start.y, 0.]));
        last_point = segment.to();
    }
    nurbs.controls.push(Model::Vector(vec![last_point.x, last_point.y, 0.]));
    nurbs
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center: Box<Model>, 
    pub radius: f32,
}

impl Circle {
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        get_polyline(self.get_path(), tolerance)
    }
    pub fn get_path(&self) -> lyon::path::Path {
        let center = get_point(&*self.center);
        let mut builder = lyon::path::Path::builder();
        builder.add_circle(center, self.radius, Winding::Positive); 
        builder.build()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    Box<Model>, 
    pub max:    Box<Model>, 
    pub radius: f32,
}

impl Rectangle {
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        get_polyline(self.get_path(), tolerance)
    }
    pub fn get_path(&self) -> lyon::path::Path { 
        let min = get_point(&*self.min);
        let max = get_point(&*self.max);
        let mut builder = lyon::path::Path::builder();
        builder.add_rounded_rectangle(
            &Box2D {min, max},
            &BorderRadii {
                top_left:     self.radius,
                top_right:    self.radius,
                bottom_left:  self.radius,
                bottom_right: self.radius,
            },
            Winding::Positive,
        );
        builder.build()
    }
}

fn get_point(model: &Model) -> Point {
    match model {
        Model::Vector(m) => point(m[0], m[1]),
        _ => point(0., 0.),
    }
}

fn get_vector(model: &Model) -> Vector {
    match model {
        Model::Vector(m) => vector(m[0], m[1]),
        _ => vector(0., 0.),
    }
}

fn get_polyline(path: lyon::path::Path, tolerance: f32) -> Vec<f32> {
    let mut polyline = vec![];
    for event in path.iter().flattened(tolerance){
        match event {
            Event::Begin{at}     => {
                polyline.extend(at.to_array()); 
                polyline.push(0.);
            },
            Event::Line{from:_, to} => {
                polyline.extend(to.to_array()); 
                polyline.push(0.);
            },
            Event::End{last:_, first, close} => {
                if close { 
                    polyline.extend(first.to_array()); 
                    polyline.push(0.);
                }
            },
            _ => (),
        }
    };
    polyline
}

fn build_path_from_iterator<I>(path_iterator: I) -> lyon::path::Path
where
    I: Iterator<Item = PathEvent>,
{
    let mut builder = lyon::path::Path::builder();
    for event in path_iterator {
        match event {
            PathEvent::Begin { at } => {
                builder.begin(at);
            }
            PathEvent::Line{from:_, to} => {
                builder.line_to(to);
            }
            PathEvent::Quadratic { from:_, ctrl, to } => {
                builder.quadratic_bezier_to(ctrl, to);
            }
            PathEvent::Cubic { from:_, ctrl1, ctrl2, to }=> {
                builder.cubic_bezier_to(ctrl1, ctrl2, to );
            }
            PathEvent::End { last:_, first:_, close } => {
                builder.end(close);
            }
        }
    }
    builder.build()
}



// pub fn get_vector_at_t(path: &lyon::path::Path, t: f32) -> Vec<f32> {//Option<Point> {

//     let mut iterator = path.into_iter().flattened(0.05); // Adjust the flattening tolerance as needed
    
//     let mut accumulated_length = 0.;
//     let mut total_length = 0.;

//     while let Some(segment) = iterator.next() {
//         let start = segment.from();
//         let end = segment.to();
//         total_length += (start - end).length();
//         //log("raw calc vector at t from path!!");
//     }

//     iterator = path.into_iter().flattened(0.05);

//     while let Some(segment) = iterator.next() {
//         let start = segment.from();
//         let end = segment.to();
//         let segment_length = (start - end).length();
        
//         let segment_t = accumulated_length / total_length;
//         console_log!("t, segment_t = {}, {}", t, segment_t);
//         if t <= segment_t {
//             let [x,y] = (start + (end - start) * (t / segment_t)).to_array(); //Some(start + (end - start) * (t / segment_t));
//             console_log!("get_vector_at_t = {}, {}", x, y);
//             return vec![x, y, 0.];
//         }
        
//         accumulated_length += segment_length;
//     }
//     vec![0., 0., 0.]
//     //let crap = path.last_endpoint().unwrap_or_default();
//     //Some() // Return the last point if t is greater than 1.0
// }