//use super::mesh::Mesh;
use lyon::math::{Box2D, point};
use lyon::path::{Event, Path, Winding, builder::BorderRadii};
use lyon::path::path::Builder;
use lyon::path::traits::PathIterator;
//use lyon::tessellation::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Path2D::default")]
pub struct Path2D {
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

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Rectangle::default")]
pub struct Rectangle {
    pub min:    [f32; 2],
    pub max:    [f32; 2],
    pub radius:  f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Circle::default")]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,
}

impl Path2D { // Polyline for  
    pub fn get_polyline(&self, tolerance: f32) -> Vec<f32> {
        //let path = self.get_builder().build();
        let mut builder = Path::builder();
        self.add_parts_to_builder(&mut builder);
        let mut polyline = vec![];
        for event in builder.build().iter().flattened(tolerance){
            match event {
                Event::Begin { at } => {polyline.extend(at.to_array()); polyline.push(0.);},
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
        for part in self.parts.clone(){
            match part {
                Part::Begin(p)  => {builder.begin(point(p[0], p[1])); ()}, // polyline.extend(p); polyline.push(0.);
                Part::LineTo(p) => {builder.line_to(point(p[0], p[1])); ()},
                //Part::ArcTo(p)  => {builder.add_rounded_rectangle(rect, radii, winding)},
                Part::End(b)    =>  builder.end(b),
                Part::Rectangle(rect) => self.add_rounded_rectangle(builder, rect),
                Part::Circle(c)  => builder.add_circle(point(c.center[0], c.center[1]), c.radius, Winding::Positive),
            };
        }
        //builder
    }
    fn add_rounded_rectangle(&self, builder: &mut Builder, rect: Rectangle) {
        builder.add_rounded_rectangle(
            &Box2D { min: point(rect.min[0], rect.min[1]), max: point(rect.max[0], rect.max[1]) },
            &BorderRadii {
                top_left:     rect.radius,
                top_right:    rect.radius,
                bottom_left:  rect.radius,
                bottom_right: rect.radius,
            },
            Winding::Positive,
        );
    }
}

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