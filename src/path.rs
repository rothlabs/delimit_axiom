use super::Model;
use lyon::math::{Box2D, Angle, Vector, vector, Point, point};
use lyon::path::{Event, Winding, ArcFlags, PathEvent};
use lyon::path::builder::BorderRadii;
use lyon::path::traits::{PathIterator, SvgPathBuilder};
use serde::{Deserialize, Serialize};
use glam::*;

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
    pub fn get_paths(&self) -> Vec<lyon::path::Path> {
        match self {
            Model::Group(m) => m.get_paths(), 
            _ => vec![self.get_path()],//lyon::path::PathSlice::default()
        }
    }
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
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        get_polyline(self.get_path().clone(), tolerance)
    }
    pub fn get_path(&self) -> lyon::path::Path { 
        let mut builder = lyon::path::Path::builder();
        for part in &self.parts {
            for path in part.get_paths(){
                builder.extend_from_paths(&[path.as_slice()]);
            }
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