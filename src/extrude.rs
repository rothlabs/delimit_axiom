use crate::{nurbs::Nurbs, Area, Circle, Shape, Shapes, Model, ModelsToShapes, Rectangle, Reshape
};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)] //  = "Extrude::default"
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
    pub fn get_shapes(&self) -> Vec<Shape> {
        let shapes0 = self.parts.shapes();
        if self.length == 0. {
            return shapes0;
        }
        //let axis = self.axis;//get_vec3_or(&self.axis, Vec3::Z).normalize(); 
        let basis = ExtrudeBasis::new(self.axis * self.length);
        let high_rank = shapes0.high_rank();
        let mut shapes1 = shapes0.clone();
        for shape0 in shapes0 {
            let mut shape1 = shape0.clone();
            shape1.invert().reshape(basis.mat4);
            shapes1.push(shape1.clone());
            if shape0.rank < high_rank {
                let mut shape2 = basis.nurbs.shape();
                shape2.controls = vec![shape0.clone(), shape1];
                shape2.validate(); 
                shapes1.push(shape2);
            }
        }
        self.reshape.get_reshapes(shapes1) 
    }
    pub fn from_area(area: Area, length: f32, reshape: &Reshape) -> Self {
        let mut model = Self::default();
        model.parts = vec![Model::Area(area)];
        model.length = length;
        model.reshape = reshape.clone();
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
                sign:    1.,
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
    pub reshape: Reshape,
}

impl Cuboid {
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut rect = Rectangle::default();
        rect.lengths = [self.lengths[0], self.lengths[1]];
        let area = Area::from_part(Model::Rectangle(rect));
        Extrude::from_area(area, self.lengths[2], &self.reshape).get_shapes()
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
    pub fn get_shapes(&self) -> Vec<Shape> {
        let mut circle = Circle::default();
        circle.radius = self.radius;
        circle.center = self.center;
        let area = Area::from_part(Model::Circle(circle));
        Extrude::from_area(area, self.length, &self.reshape).get_shapes()
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