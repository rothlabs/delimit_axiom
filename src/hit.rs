mod basis3;
mod shader;
mod shader_parts;
mod traced;
pub mod hit2;
pub mod hit3;

use crate::CurveShape;


#[derive(Clone)]
pub struct TestPair3 {
    pub group: usize,
    pub i0: usize,
    pub i1: usize,
    pub reverse: bool,
}

#[derive(Clone)]
pub struct Hit3 {
    pub group: usize,
    pub i0: usize,
    pub i1: usize,
    pub curve0: CurveShape,
    pub curve1: CurveShape,
    pub curve2: CurveShape,
}

#[derive(Clone)]
pub struct Miss3 {
    pub group: usize,
    pub i0: usize,
    pub i1: usize,
    pub distance: f32,
    pub dot0: f32,
    pub dot1: f32,
}

#[derive(Clone)]
pub struct Miss {
    pub distance: f32,
    pub dot: f32,
}

#[derive(Clone)]
pub struct MissPair {
    pub index: TestPair3,
    pub distance: f32,
    pub dot0: f32,
    pub dot1: f32,
}