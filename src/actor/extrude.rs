use glam::*;
use crate::shape::*;

pub trait ToExtrude {
    fn extrude(self) -> Extrude;
}

impl ToExtrude for Vec<Shape> {
    fn extrude(self) -> Extrude {
        Extrude {
            shapes: self,
            axis:   Vec3::Z,
            length: 1.,
            anchor: 0.,
        }
    }
}


pub struct Extrude {
    pub shapes:  Vec<Shape>,
    pub axis:    Vec3,
    pub length:  f32,
    pub anchor:  f32
}


// TODO: two types of extrude: 1. reshape all with translation by length and invert originals and increament lower ranks up one over length. 
//                     2. Same as 1 but also create new shape with highest ranking plus one and remaining one lower ranks copied into its boundaries 
impl Extrude {
    pub fn shapes(&self) -> Vec<Shape> {
        let shapes0 = self.shapes.reshaped(Mat4::from_translation(vec3(0., 0., -self.anchor * self.length)));
        if self.length == 0. {
            return shapes0;
        }

        //let axis = self.axis;//get_vec3_or(&self.axis, Vec3::Z).normalize(); 
        let vector = self.axis * self.length;
        let basis = ExtrudeBasis::new(vector);
        //let high_rank = shapes0.high_rank();
        let mut shapes1 = vec![]; //shapes0.clone();
        for shape0 in &shapes0 {
            let shape1 = shape0.reshaped(basis.mat4);
            //shape1.invert().reshape(basis.mat4);
            shapes1.push(shape0.inverted());
            shapes1.push(shape1.clone());
            if shape0.boundaries.is_empty() {// if shape0.rank < high_rank {
                let mut shape2 = basis.nurbs.shape();
                shape2.controls = vec![shape0.clone(), shape1];
                shape2.validate(); 
                shapes1.push(shape2);
            }
        }
        shapes1
    }
    pub fn axis(&mut self, axis: Vec3) -> &mut Self {
        self.axis = axis;
        self
    } 
    pub fn length(&mut self, length: f32) -> &mut Self {
        self.length = length;
        self
    } 
    pub fn anchor(&mut self, anchor: f32) -> &mut Self {
        self.anchor = anchor;
        self
    } 
}

struct ExtrudeBasis {
    nurbs: Basis,
    mat4: Mat4,
}

impl ExtrudeBasis {
    fn new(translation: Vec3) -> Self {
        Self {
            nurbs: Basis {
                sign:    1.,
                order:   2,
                min: 0.,
                max: 1.,
                knots:   vec![0., 0., 1., 1.],
                weights: vec![1., 1.],
            },
            mat4: Mat4::from_translation(translation),
        }
    }
}


    // pub fn from_area(area: Area, length: f32, reshape: &Reshape) -> Self {
    //     let mut model = Self::default();
    //     model.shapes = vec![Model::Area(area)];
    //     model.length = length;
    //     model.reshape = reshape.clone();
    //     model
    // }

// #[derive(Clone, Default, Serialize, Deserialize)]
// #[serde(default)]
// pub struct Cuboid {
//     pub lengths: [f32; 3],
//     pub reshape: Reshape,
// }

// impl Cuboid {
//     pub fn shapes(&self) -> Vec<Shape> {
//         let mut rect = Rectangle::default();
//         rect.lengths = [self.lengths[0], self.lengths[1]];
//         let area = Model::Rectangle(rect).shapes().area();//Area::from_part(Model::Rectangle(rect));
//         //Extrude::from_area(area, self.lengths[2], &self.reshape).shapes()
//     }
// }

// #[derive(Clone, Default, Serialize, Deserialize)]
// #[serde(default = "Cylinder::default")]
// pub struct Cylinder {
//     pub radius: f32,
//     pub length: f32,
//     pub center: Vec2,//[f32; 2],
//     pub reshape: Reshape,
// }

// impl Cylinder {
//     pub fn shapes(&self) -> Vec<Shape> {
//         let mut circle = Circle::default();
//         circle.radius = self.radius;
//         circle.center = self.center;
//         let area = Model::Circle(circle).shapes().area();
//         //Extrude::from_area(area, self.length, &self.reshape).shapes()
//     }
// }






// impl Default for Extrude {
//     fn default() -> Self {
//         Self {
//             shapes:   vec![],
//             axis:    Vec3::Z,
//             length:  1.,
//         }
//     }
// }







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