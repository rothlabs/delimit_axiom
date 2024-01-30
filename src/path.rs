use super::{Model, log};
use lyon::math::{Transform, Box2D, Point, point};
use lyon::path::{Event, Winding, builder::BorderRadii};
use lyon::path::path::Builder;
use lyon::path::traits::PathIterator;
use lyon::geom::Arc;
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Path::default")]
pub struct Path {
    pub parts: Vec<Part>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Part {
    Begin([f32; 2]),
    LineTo([f32; 2]),
    //ArcTo([f32; 2]),
    End(bool),
    Rectangle(Rectangle),
    Circle(Circle),
}

impl Path { // Polyline for  
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        //let path = self.get_builder().build();
        let mut builder = lyon::path::Path::builder();
        self.add_parts_to_builder(&mut builder);
        let mut polyline = vec![];
        for event in builder.build().iter().flattened(tolerance){
            match event {
                Event::Begin { at }     => {polyline.extend(at.to_array()); polyline.push(0.);},
                Event::Line{from:_, to} => {polyline.extend(to.to_array()); polyline.push(0.);},
                Event::End{last:_, first, close } => {
                    if close { polyline.extend(first.to_array()); polyline.push(0.);}
                },
                _ => (),
            }
        };
        polyline
    }
    pub fn add_parts_to_builder(&self, builder: &mut Builder) { 
        //let mut builder = Path::builder();
        for part in &self.parts {
            match part {
                Part::Begin(p)     => {builder.begin(point(p[0], p[1])); ()}, // polyline.extend(p); polyline.push(0.);
                Part::LineTo(p)    => {builder.line_to(point(p[0], p[1])); ()},
                //Part::ArcTo(p)  => {builder.add_rounded_rectangle(rect, radii, winding)},
                Part::End(p)       =>  builder.end(*p),
                Part::Circle(p)    =>  p.add_self_to_builder(builder),//builder.add_circle(point(c.center[0], c.center[1]), c.radius, Winding::Positive),
                Part::Rectangle(p) =>  p.add_self_to_builder(builder),// self.add_rounded_rectangle(builder, rect),
            };
        }
        //builder
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center: Box<Model>,//Box<Model>,//[f32; 2],
    pub radius: f32,
}

impl Circle {
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        let path = Path{parts: vec![Part::Circle(self.clone())]};
        path.get_polyline(tolerance)
    }
    pub fn add_self_to_builder(&self, builder: &mut Builder){
        let center = get_point_from_model(&*self.center);
        builder.add_circle(center, self.radius, Winding::Positive); // point(self.center[0], self.center[1])
        //let mat4 = Mat4::IDENTITY;
        //mat4.
        //Transform::new(m11, m12, m21, m22, m31, m32)
        //builder.transformed(transform)
        //Arc::circle(center, radius).
        log("circle: add self!");
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    [f32; 2],
    pub max:    [f32; 2],
    pub radius:  f32,
}

impl Rectangle {
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        let path = Path{parts: vec![Part::Rectangle(self.clone())]};
        path.get_polyline(tolerance)
    }
    pub fn add_self_to_builder(&self, builder: &mut Builder) { // , rect: Rectangle
        builder.add_rounded_rectangle(
            &Box2D { min: point(self.min[0], self.min[1]), max: point(self.max[0], self.max[1]) },
            &BorderRadii {
                top_left:     self.radius,
                top_right:    self.radius,
                bottom_left:  self.radius,
                bottom_right: self.radius,
            },
            Winding::Positive,
        );
    }
}

fn get_point_from_model(model: &Model) -> Point {
    match model {
        Model::Vector(m) => point(m[0], m[1]),
        _ => point(0., 0.),
    }
}


        // let center = match &*self.center {
        //     Model::Vector(m) => point(m[0], m[1]),
        //     _ => point(0., 0.),
        // };

// pub fn get_builder(&self) -> Builder { 
//     let mut builder = Path::builder();
//     for part in self.parts.clone(){
//         match part {
//             Part::Begin(p)  => {builder.begin(point(p[0], p[1])); ()}, // polyline.extend(p); polyline.push(0.);
//             Part::LineTo(p) => {builder.line_to(point(p[0], p[1])); ()},
//             //Part::ArcTo(p)  => {builder.add_rounded_rectangle(rect, radii, winding)},
//             Part::End(b)    =>  builder.end(b),
//             Part::Rectangle(rect) => self.add_rounded_rectangle(&mut builder, rect),
//             Part::Circle(c)  => builder.add_circle(point(c.center[0], c.center[1]), c.radius, Winding::Positive),
//         };
//     }
//     builder
// }