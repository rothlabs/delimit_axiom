use glam::*;
use serde::{Deserialize, Serialize};
use crate::actor::{MakeArea, ToExtrude};
use crate::shape::*;
use crate::{Area, Circle, Model, Models, Rectangle, Reshape};



#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] 
pub struct Extrude {
    pub parts:   Vec<Model>,
    pub reshape: Reshape,
    pub axis:    Vec3,//[f32; 3],
    pub length:  f32,
}

impl Default for Extrude {
    fn default() -> Self {
        Self {
            parts:   vec![],
            reshape: Reshape::default(),
            axis:    Vec3::Z,
            length:  1.,
        }
    }
}


// TODO: two types of extrude: 1. reshape all with translation by length and invert originals and increament lower ranks up one over length. 
//                     2. Same as 1 but also create new shape with highest ranking plus one and remaining one lower ranks copied into its boundaries 
impl Extrude {
    pub fn shapes(&self) -> Vec<Shape> {
        self.reshape.shapes(
            self.parts.shapes()
                .extrude()
                    .axis(self.axis)
                    .length(self.length)
                    .shapes()
        )
    }
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Cuboid {
    pub lengths: [f32; 3],
    pub reshape: Reshape,
}

impl Cuboid {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut rect = Rectangle::default();
        rect.lengths = [self.lengths[0], self.lengths[1]];
        let shapes = Model::Rectangle(rect).shapes().area().extrude().length(self.lengths[2]).shapes();
        self.reshape.shapes(shapes)
        //let area = Model::Rectangle(rect).shapes().area();//Area::from_part(Model::Rectangle(rect));
        //Extrude::from_area(area, self.lengths[2], &self.reshape).shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Cylinder::default")]
pub struct Cylinder {
    pub radius: f32,
    pub length: f32,
    pub center: Vec2,//[f32; 2],
    pub reshape: Reshape,
}

impl Cylinder {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut circle = Circle::default();
        circle.radius = self.radius;
        circle.center = self.center;
        let shapes = Model::Circle(circle).shapes().area().extrude().length(self.length).shapes();
        self.reshape.shapes(shapes)
        //let area = Model::Circle(circle).shapes().area();
        //Extrude::from_area(area, self.length, &self.reshape).shapes()
    }
}


// match &shape.rank {
//     0 => {
//         // let mut curve = CurveShape {
//         //     nurbs: basis.nurbs.clone(),
//         //     controls: vec![*point, get_reshaped_point(point, basis.mat4)], 
//         //     min: 0.,
//         //     max: 1.,
//         // };
//         let mut curve = CurveShape::from_nurbs_and_controls(
//             basis.nurbs.clone(), 
//             vec![*point, get_reshaped_point(point, basis.mat4)]
//         );
//         if self.length < 0. {
//             curve.controls.reverse();
//         }
//         shapes.push(Shape::Curve(curve));
//         shapes.push(shape.get_reshape(basis.mat4));
//     },
//     Shape::Curve(curve) => {
//         let mut facet = FacetShape {
//             nurbs: basis.nurbs.clone(),
//             controls:   vec![curve.clone(), curve.reshaped(basis.mat4)], 
//             boundaries: Rectangle::unit(),
//         };
//         if self.length < 0. {
//             facet.controls.reverse();
//         }
//         shapes.push(Shape::Facet(facet));
//         shapes.push(shape.get_reshape(basis.mat4));
//     },
//     Shape::Facet(facet) => {
//         if self.length > 0. {
//             shapes.push(Shape::Facet(facet.get_reshape(basis.mat4)));
//         }else{
//             shapes.push(Shape::Facet(facet.get_reverse_reshape(basis.mat4))); 
//         }
//     },
// }