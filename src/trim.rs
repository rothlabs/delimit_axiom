use crate::{hit::Miss, CurveHit, CurveShape, HitTester2, Shape, Spatial3};
use glam::*;

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

pub struct Trim {
    pub tester: HitTester2,
    pub group:  Vec<CurveShape>,
    pub hits:   Vec<Vec<CurveHit>>, 
    pub miss:   Vec<Vec<Miss>>, 
    pub curves: Vec<CurveShape>,
    pub shapes: Vec<Shape>,
}

impl Trim { 
    pub fn new(curves0: Vec<CurveShape>, tolerance: f32) -> Self {
        let duplication_tolerance = tolerance * 5.; 
        Trim {
            tester: HitTester2 {
                curves: (CurveShape::default(), CurveShape::default()),
                spatial: Spatial3::new(duplication_tolerance), 
                points:  vec![],
                tolerance,
                duplication_tolerance,
            },
            hits: vec![vec![]; curves0.len()],
            miss: vec![vec![]; curves0.len()],
            group: curves0,
            curves: vec![],
            shapes: vec![],
        }
    }

    pub fn build(&mut self) -> Vec<CurveShape> {
        self.test_groups();
        for i in 0..self.group.len() {
            if self.hits[i].is_empty() {
                // self.miss[i] = self.miss[i].clone().into_iter().filter(
                //     |a| !a.distance.is_nan() && !a.dot.is_nan() && a.dot.abs() > 0.01
                // ).collect();
                // self.miss[i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                // if self.miss[i].is_empty() || self.miss[i][0].dot > -0.01 {
                    self.curves.push(self.group[i].clone());   
                //}
            }else{
                self.hits[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
                self.add_bounded_curves(i);   
            }
        }
        self.curves.clone()
    }

    fn add_bounded_curves(&mut self, i: usize) {
        let mut curve = self.group[i].clone();
        let min_basis = curve.min;
        let mut start_k = 0;
        let mut set_min = false;
        //let mut use_hits = false;
        for k in 0..self.hits[i].len() {
            if self.hits[i][k].u > 0.01 && self.hits[i][k].u < 0.99 {
                start_k = k;
                //use_hits = true;
                break;
            }
        }
        //if use_hits {
            //let first = &self.hits[i][start_k];//.first().unwrap();
            if self.hits[i][start_k].dot > 0. {set_min = true;} //  * curve.nurbs.sign
            for curve_hit in self.hits[i].iter().skip(start_k) { //for curve_hit in &self.hits[i] { 
                if curve_hit.u < 0.99 {
                    if set_min {
                        curve.set_min(curve_hit.u);
                    }else{
                        curve.set_max(min_basis, curve_hit.u);
                        self.curves.push(curve);
                        curve = self.group[i].clone();
                    }
                    set_min = !set_min;
                }
            }
        //}
        if !set_min {
            self.curves.push(curve);
        }
    }

    fn test_groups(&mut self){
        for i0 in 0..self.group.len() {
            for i1 in i0..self.group.len() {
                if i0 == i1 {continue}
                self.tester.curves.0 = self.group[i0].clone();
                self.tester.curves.1 = self.group[i1].clone();
                for u0 in self.group[i0].get_unique_knots() { // vec![0., 0.25, 0.5, 0.75, 1.] { //
                    for u1 in self.group[i1].get_unique_knots() { // vec![0., 0.25, 0.5, 0.75, 1.] { //
                        self.test_curves(i0, i1, u0, u1);
                    }
                }
            }
        }
    }

    fn test_curves(&mut self, i0: usize, i1: usize, u0: f32, u1: f32) { 
        match self.tester.test(u0, u1) {
            Ok(hit) => {
                self.hits[i0].push(hit.hit.0);
                self.hits[i1].push(hit.hit.1);
                //self.shapes.push(Shape::Point(hit.center));
            },
            Err(miss) => {
                //self.shapes.push(Shape::Point(miss.0.point.clone()));
                //self.shapes.push(Shape::Point(miss.1.point.clone()));
                self.miss[i0].push(miss.0);
                self.miss[i1].push(miss.1);
            }
        }
    }
}


// let mut miss_dot = 0.;
                    // if !self.miss[g][i].is_empty() {
                    //     let mut count = 1.;
                    //     miss_dot = self.miss[g][i][0].dot;
                    //     console_log!("original dot: {}", self.miss[g][i][0].dot);
                    //     for miss in self.miss[g][i].iter().skip(1) {
                    //         if !miss.dot.is_nan() && miss.point.distance(self.miss[g][i][0].point) < self.tester.tolerance {
                    //             miss_dot += miss.dot;
                    //             count += 1.;
                    //         }
                    //     }
                    //     miss_dot /= count;
                    //     console_log!("new dot: {}", miss_dot);
                    // }



// fn add_bounded_curves(&mut self, g: usize, i: usize) {
//     let mut curve = self.groups[g][i].clone();
//     let min_basis = curve.min;
//     let first = self.hits[g][i].first().unwrap();
//     let mut set_min = false;
//     if first.dot > 0. {set_min = true;} //  * curve.nurbs.sign
//     for curve_hit in &self.hits[g][i] { 
//         if set_min {
//             curve.set_min(curve_hit.u);
//         }else{
//             curve.set_max(min_basis, curve_hit.u);
//             self.curves.push(curve);
//             curve = self.groups[g][i].clone();
//         }
//         set_min = !set_min;
//     }
//     if !set_min {
//         self.curves.push(curve);
//     }
// }




        //console_log!("try face pairs: {}, {}", self.grouped_facets.len(), self.grouped_facets.len());
        //let start = Instant::now();
                //let elapsed = start.elapsed();
        //console_log!("timed: {:?}", elapsed);




// pub fn get_shapes(&mut self) -> Vec<Shape> {
//     let spatial = self.set_samples_and_get_spatial();
//     self.clear_params();
//     self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_curve_param);
//     self.reduce_cell_and_step();
//     let spatial = self.set_samples_and_get_spatial();
//     self.for_spatial_pairs(&spatial, &mut UnionBasis2::add_intersection);
//     for i in 0..self.curves.len() {
//         self.hits[i].sort_by(|a, b| a.u.partial_cmp(&b.u).unwrap());
//         if self.hits[i].is_empty() {
//             self.shapes.push(Shape::Curve(self.curves[i].clone()));
//             continue;
//         }
//         self.add_split_curves(i);
//     }
//     self.shapes.clone()
// }

// fn add_split_curves(&mut self, i: usize) {
//     let first = self.hits[i].first().unwrap();
//     let mut set_min = false;
//     if first.angle > 0. {set_min = true;}
//     let mut curve = self.curves[i].clone();
//     for itc in self.get_merged_hits(i, first) { 
//         self.shapes.push(Shape::Point(vec3(itc.point.x, itc.point.y, 0.)));
//         if set_min {
//             curve.min = itc.u;
//         }else{
//             curve.max = itc.u;
//             self.shapes.push(Shape::Curve(curve));
//             curve = self.curves[i].clone();
//         }
//         set_min = !set_min;
//     }
//     if !set_min {
//         self.shapes.push(Shape::Curve(curve));
//     }
// }

// fn get_merged_hits(&self, i: usize, first: &Hit2) -> Vec<Hit2> {
//     let mut point = first.point;
//     let mut intersections = vec![first.clone()];
//     for itc in &self.hits[i] {
//         if itc.point.distance(point) > self.cell_size {
//             intersections.push(itc.clone());
//         }
//         point = itc.point;
//     }
//     intersections
// }

// fn clear_params(&mut self) {
//     for i in 0..self.curves.len() {
//         if let Some(cr) = self.curve_params.get_mut(&i) {
//             cr.params.clear();
//         }
//     }
// }

// fn add_curve_param(&mut self, curve_index0: usize, _c1: usize, u0: f32, _u1: f32) {
//     if let Some(cr) = self.curve_params.get_mut(&curve_index0) {
//         cr.params.push(u0);
//     }
// }

// fn add_intersection(&mut self, curve_index0: usize, curve_index1: usize, u0: f32, u1: f32) {
//     if let Some(itc) = self.get_hit(&curve_index0, &curve_index1, u0, u1) {
//         if 0.01 < itc.u && itc.u < 0.99 {
//             self.hits[curve_index0].push(itc.clone());
//         } 
//     }
// }

// fn for_spatial_pairs<F>(&mut self, spatial: &Spatial2, func: &mut F) 
// where F: FnMut(&mut UnionBasis2, usize, usize, f32, f32)  { 
//     spatial.for_pairs(&mut |i0: usize, i1: usize| { 
//         let Sample2 {index: c0, point: p0, u: u0} = self.samples[i0];
//         let Sample2 {index: c1, point: p1, u: u1} = self.samples[i1];
//         if c0 == c1 {return}
//         if p0.distance(p1) > self.cell_size {return}
//         func(self, c0, c1, u0, u1);
//     });
// }


// fn reduce_cell_and_step(&mut self) {
//     for i in 0..self.curves.len() {
//         if let Some(cr) = self.curve_params.get_mut(&i) {
//             cr.params.sort_by(|a, b| a.partial_cmp(b).unwrap());
//             if cr.params.is_empty() {continue}
//             let mut filled = vec![cr.params[0]];
//             for uu in cr.params.windows(2) {
//                 if uu[1] - uu[0] <= cr.step + EPSILON {
//                     for k in 1..20 {
//                         let fill_u = uu[0] + k as f32 * (cr.step/10.);
//                         if fill_u >= uu[1] {break}
//                         filled.push(fill_u);
//                     }
//                 }
//                 filled.push(uu[1]);
//             }
//             cr.params = filled;
//             cr.step /= 10.
//         }
//     }
//     self.cell_size /= 10.;
// }


// fn set_samples_and_get_spatial(&mut self) -> Spatial2 { 
//     let mut spatial: Spatial2 = Spatial2::new(self.cell_size); 
//     self.samples.clear();
//     for (_, CurveParams {i, params, ..}) in &self.curve_params { 
//         for u in params {
//             let point = self.curves[*i].get_vec2_at_u(*u);
//             self.samples.push(Sample2 {
//                 index: *i,
//                 point,
//                 u: *u,
//             });
//             spatial.insert(&point, self.samples.len()-1);
//         }
//     }
//     spatial
// }



//for_merged_intersections(&self.intersections[i].clone(), self.cell_size, &mut |itc: &Intersection2| { 

// fn for_merged_intersections<F>(intersections: &Vec<Intersection2>, tolerance: f32, func: &mut F)// -> Vec<Intersection2> 
// where F: FnMut(&Intersection2)  {
//     let mut point = intersections.first().unwrap().point;
//     //let mut intersections = vec![first.clone()];
//     for itc in intersections{
//         if itc.point.distance(point) > tolerance {
//             func(&itc);
//             //intersections.push(itc.clone());
//         }
//         point = itc.point;
//     }
//     //intersections
// }