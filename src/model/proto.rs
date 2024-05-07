use glam::*;
use serde::*;
use crate::shape::*;
use crate::actor;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Circle {
    pub center:  Vec2, 
    pub radius:  f32,
    pub reverse: bool,
    pub arrows: usize,
}

impl Circle {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut shapes = actor::circle()
            .center(self.center)
            .radius(self.radius)
            .shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
                    if self.arrows > 0 {
                        for i in 0..self.arrows {
                            let mut curve = Shape::default();
                            let arrow = shapes[0].get_arrow(&[i as f32 / (self.arrows - 1) as f32]);
                            curve.controls.push(rank0(arrow.point));
                            curve.controls.push(rank0(arrow.point + arrow.delta));
                            curve.validate();
                            shapes.push(curve);
                        }
                    }
        shapes
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Rectangle {
    pub half_lengths: [f32; 2],
    pub lengths:      [f32; 2],
    pub point_a:      [f32; 2], 
    pub point_b:      [f32; 2], 
    pub radius:       f32,
    pub reverse:      bool,
}

impl Rectangle {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut point_a = -Vec2::from_array(self.half_lengths);
        let mut point_b = -point_a;
        if self.lengths[0] > 0. || self.lengths[1] > 0. {
            point_a = -Vec2::from_array(self.lengths) / 2.;
            point_b = -point_a;
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        let mut shapes = actor::rectangle()
            .points(point_a, point_b)
            .radius(self.radius)
            .shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
        shapes
    }
}


#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Slot {
    pub length:      f32,
    pub half_length: f32,
    pub point_a:    [f32; 2], 
    pub point_b:    [f32; 2], 
    pub radius:      f32,
    pub reverse:     bool,
}

impl Slot {
    pub fn shapes(&self) -> Vec<Shape> {
        let mut point_a = vec2(-self.half_length, 0.);
        let mut point_b = vec2( self.half_length, 0.);
        if self.length > 0. {
            point_a = vec2(-self.length/2., 0.);
            point_b = vec2(self.length/2., 0.);
        }else if self.point_a[0] > 0. || self.point_a[1] > 0. || self.point_b[0] > 0. || self.point_b[1] > 0. {
            point_a = Vec2::from_array(self.point_a);
            point_b = Vec2::from_array(self.point_b);
        }
        let mut shapes = actor::slot()
            .points(point_a, point_b)
            .radius(self.radius)
            .shapes();
        if self.reverse {
            shapes.reverse_direction();
        }
        shapes
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Arc {
    pub center: [f32; 2], 
    pub radius: f32,
    pub angle_a: f32,
    pub angle_b: f32,
    pub point_a: f32,
    pub point_b: f32,
    pub point_c: f32,
    //pub reverse: bool,
}

impl Arc {
    pub fn shapes(&self) -> Vec<Shape> {
        // let mut revolve = Revolve {
        //     parts: vec![Model::Point([self.center[0] + self.radius, self.center[1], 0.])],
        //     center: [self.center[0], self.center[1], 0.],
        //     axis: [0., 0., 1.],
        //     angle: self.angle,
        //     transform: Group::default(),
        // };
        // //revolve.transform.reverse = self.reverse;
        // revolve.get_shapes()
        vec![]
    }
}