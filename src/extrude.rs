use crate::{get_reshaped_point, get_shapes, get_vec3_or, nurbs::Nurbs, Arc, Area, Circle, CurveShape, FacetShape, Group, Model, Rectangle, Shape};
use lyon::algorithms::length;
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Extrude::default")]
pub struct Extrude {
    pub parts:  Vec<Model>,
    pub axis:   [f32; 3],
    pub length: f32,
    pub transform: Group,
}

impl Extrude {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let axis = get_vec3_or(&self.axis, Vec3::Z).normalize(); 
        let basis = ExtrudeBasis::new(axis * self.length);
        let mut shapes = vec![];
        for shape in get_shapes(&self.parts) {
            if let Shape::Facet(facet) = &shape { 
                if self.length > 0. {
                    shapes.push(Shape::Facet(facet.get_reverse_reshape(Mat4::IDENTITY)));
                }else{
                    shapes.push(shape.clone());
                }
            }else{
                shapes.push(shape.clone());
            }
            match &shape {
                Shape::Point(point) => {
                    let mut curve = CurveShape {
                        nurbs: basis.nurbs.clone(),
                        controls: vec![get_reshaped_point(point, basis.mat4), *point], 
                        min: 0.,
                        max: 1.,
                    };
                    if self.length < 0. {
                        curve.controls.reverse();
                    }
                    //curve.controls.push(*point);
                    //if self.length > 0. {
                        shapes.push(Shape::Curve(curve));
                        shapes.push(shape.get_reshape(basis.mat4));
                    // }else{
                    //     shapes.push(Shape::Curve(curve).get_reshape(basis.mat4));
                    //     shapes.push(shape);
                    // }
                },
                Shape::Curve(curve) => {
                    let mut facet = FacetShape {
                        nurbs: basis.nurbs.clone(),
                        controls:   vec![curve.get_reshape(basis.mat4), curve.clone()], 
                        boundaries: vec![],
                        perimeter:  false,
                    };
                    if self.length < 0. {
                        facet.controls.reverse();
                    }
                    //facet.controls.push(curve.clone()); 
                    //if self.length > 0. {
                        shapes.push(Shape::Facet(facet));
                        shapes.push(shape.get_reshape(basis.mat4));
                    // }else{
                    //     shapes.push(Shape::Facet(facet).get_reshape(basis.mat4));
                    //     shapes.push(shape);
                    // }
                },
                Shape::Facet(facet) => {
                    if self.length > 0. {
                        //shapes.push(shape.clone());
                        shapes.push(Shape::Facet(facet.get_reshape(basis.mat4)));
                    }else{
                        shapes.push(shape.get_reverse_reshape(basis.mat4));
                    }
                },
            }
        }
        self.transform.get_reshapes(shapes) 
    }
    pub fn from_area(area: Area, length: f32, transform: &Group) -> Self {
        let mut model = Self::default();
        model.parts = vec![Model::Area(area)];
        model.length = length;
        model.transform = transform.clone();
        model
    }
}

struct ExtrudeBasis {
    nurbs: Nurbs,
    mat4: Mat4,
}

impl ExtrudeBasis {
    fn new(translation: Vec3) -> Self {
        Self {
            nurbs: Nurbs {
                order:   2,
                knots:   vec![0., 0., 1., 1.],
                weights: vec![1., 1.],
            },
            mat4: Mat4::from_translation(translation),
        }
    }
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Cuboid::default")]
pub struct Cuboid {
    pub lengths: [f32; 3],
    pub transform: Group,
}

impl Cuboid {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut rect = Rectangle::default();
        rect.lengths = [self.lengths[0], self.lengths[1]];
        let area = Area::from_part(Model::Rectangle(rect));
        Extrude::from_area(area, self.lengths[2], &self.transform).get_shapes()
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Cylinder::default")]
pub struct Cylinder {
    pub radius: f32,
    pub length: f32,
    pub transform: Group,
}

impl Cylinder {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut circle = Circle::default();
        circle.radius = self.radius;
        let area = Area::from_part(Model::Circle(circle));
        Extrude::from_area(area, self.length, &self.transform).get_shapes()
    }
}


// let mut length = self.length;
        // let mut transform = self.transform;
        // if length < 0. {
        //     length = -length;
        //     transform.position = [transform.position[2]];
        // }