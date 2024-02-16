pub mod curve;
pub mod facet;

use glam::*;

#[derive(Clone)]
pub struct Nurbs {
    pub order:    usize,       // order = polynomial_degree + 1
    pub knots:    Vec<f32>,    // knot_count = order + control_count
    pub weights:  Vec<f32>,    // weight_count = control_count
}

impl Nurbs {
    pub fn get_sample_count(&self, count: usize) -> usize { 
        let mul = self.weights.len()-1;
        self.weights.len() + count * (self.order - 2) * mul
    }

    pub fn get_param_step(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> f32 {
        1. / (self.get_sample_count_with_max_distance(min_count, max_distance, controls) - 1) as f32
    }

    pub fn get_param_samples(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> Vec<f32> {
        let count = self.get_sample_count_with_max_distance(min_count, max_distance, controls);
        (0..count).map(|s| s as f32 / (count-1) as f32).collect()
    }

    pub fn get_sample_count_with_max_distance(&self, min_count: usize, max_distance: f32, controls: &Vec<Vec3>) -> usize {
        let curve = self;//.get_valid(controls.len());
        let mut distance = 0.;
        for c in controls.windows(2) {
            let dist = c[0].distance(c[1]);
            if distance < dist {distance = dist;}
        }
        let distance_based_count = (distance / max_distance).ceil() as usize;
        let mut count = min_count;
        if distance_based_count > min_count {count = distance_based_count; }
        count = count*(curve.weights.len()-1) + curve.weights.len();
        count
    }

    fn get_valid(&self, control_count: usize) -> Self {
        let order = self.order.min(control_count).max(2);
        Nurbs {
            order,//:   self.get_valid_order(control_count),
            knots:   self.get_valid_knots(control_count, order),
            weights: self.get_valid_weights(control_count),
        }
    }

    fn get_valid_weights(&self, control_count: usize) -> Vec<f32> {
        if self.weights.len() == control_count {
            self.weights.clone()
        } else {
            vec![1.; control_count]
        }
    }

    fn get_valid_knots(&self, control_count: usize, order: usize) -> Vec<f32> {
        if self.knots.len() == control_count + order { 
            self.knots.clone()
        } else {
            self.get_open_knots(control_count, order)
        }
    }

    fn get_open_knots(&self, control_count: usize, order: usize) -> Vec<f32> {
        let repeats = order - 1; // knot multiplicity = order for ends of knot vector
        let max_knot = control_count + order - (repeats * 2) - 1;
        let mut knots = vec![0_f32; repeats];
        knots.extend((0..=max_knot).map(|k| k as f32));
        knots.extend(vec![max_knot as f32; repeats]);
        knots
    }

    fn get_rational_basis_at_u(&self, u: f32) -> Vec<f32> {
        let basis = self.get_basis_at_u(u);
        let sum: f32 = self.weights.iter().enumerate().map(|(i, w)| basis[i] * w).sum();
        if sum > 0. {
            self.weights.iter().enumerate().map(|(i, w)| basis[i] * w / sum).collect()
        } else {
            vec![0.; self.weights.len()]
        }
    }

    fn get_basis_at_u(&self, normal_u: f32) -> Vec<f32> {
        let u = *self.knots.last().unwrap_or(&0.) * normal_u; // .unwrap_throw("") to js client
        let mut basis = self.get_basis_of_degree_0_at_u(u);
        for degree in 1..self.order {
            for i0 in 0..self.weights.len() {
                let i1 = i0 + 1; 
                let mut f = 0.;
                let mut g = 0.;
                if basis[i0] != 0. {
                    f = (u - self.knots[i0]) / (self.knots[degree + i0] - self.knots[i0]) 
                }
                if basis[i1] != 0. {
                    g = (self.knots[degree + i1] - u) / (self.knots[degree + i1] - self.knots[i1])
                }
                basis[i0] = f * basis[i0] + g * basis[i1];
            }
        }
        if normal_u == 1. { 
            basis[self.weights.len() - 1] = 1.; // last control edge case
        }
        basis
    }

    fn get_basis_of_degree_0_at_u(&self, u: f32) -> Vec<f32> {
        self.knots.windows(2)
            .map(|knots| {
                if u >= knots[0] && u < knots[1] {
                    1.
                } else {
                    0.
                }
            }).collect()
    }
}

impl Default for Nurbs {
    fn default() -> Self {
        Nurbs {
            order:   2,
            knots:   vec![],
            weights: vec![],    
        }
    }
}