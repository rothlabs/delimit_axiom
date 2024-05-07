pub mod curve;
pub mod facet;
pub mod proto;
pub mod reshape;
pub mod area;
pub mod extrude;
pub mod revolve;
pub mod sketch;
pub mod grid_pattern;
pub mod radial_pattern;
pub mod mirror;

pub use curve::*;
pub use facet::*;
pub use proto::*;
pub use reshape::*;
pub use area::*;
pub use extrude::*;
pub use revolve::*;
pub use sketch::*;
pub use grid_pattern::*;
pub use radial_pattern::*;
pub use mirror::*;

use glam::*;
use serde::*;
use crate::shape::*;

// #[derive(Clone, Serialize, Deserialize)] 
// pub enum Model {
//     Point(Vec3), 
//     Curve(Curve),
//     Facet(Facet),
//     Sketch(Sketch),
//     Area(Area),
//     Reshape(Reshape),
//     Arc(Arc),
//     Circle(Circle),
//     Rectangle(Rectangle),
//     Slot(Slot),
//     Extrude(Extrude),
//     Cuboid(Cuboid),
//     Cylinder(Cylinder),
//     Revolve(Revolve),
//     Union(Union),
//     GridPattern(GridPattern),
//     RadialPattern(RadialPattern),
//     Mirror(Mirror),
// }

// impl Model {
//     pub fn shapes(&self) -> Vec<Shape> {
//         match self {
//             Model::Point(m)     => vec![rank0(*m)], 
//             Model::Curve(m)     => m.shapes(),
//             Model::Facet(m)     => m.shapes(),
//             Model::Sketch(m)    => m.shapes(),
//             Model::Arc(m)       => m.shapes(),
//             Model::Circle(m)    => m.shapes(),
//             Model::Rectangle(m) => m.shapes(),
//             Model::Slot(m)      => m.shapes(),
//             Model::Reshape(m)   => m.shapes(),
//             Model::Area(m)      => m.shapes(),
//             Model::Extrude(m)   => m.shapes(),
//             Model::Cuboid(m)    => m.shapes(),
//             Model::Cylinder(m)  => m.shapes(),
//             Model::Revolve(m)   => m.shapes(),
//             Model::Union(m)     => m.shapes(),
//             Model::GridPattern(m)   => m.shapes(),
//             Model::RadialPattern(m) => m.shapes(),
//             Model::Mirror(m)        => m.shapes(),
//         }
//     }
// }

// impl Default for Model {
//     fn default() -> Self { 
//         Model::Point(Vec3::ZERO) 
//     }
// }
