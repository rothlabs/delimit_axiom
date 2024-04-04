// Function to compute basis functions and their derivatives
fn basis_functions_derivatives(t: f64, span: usize, knot_vector: &[f64], degree: usize) -> (Vec<f64>, Vec<f64>) {
    // Algorithm to compute basis functions and their derivatives using Cox-de Boor recursion formula
    let mut basis_functions = vec![0.0; degree + 1];
    let mut basis_derivatives = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    basis_functions[0] = 1.0;
    basis_derivatives[0] = 0.0;

    for j in 1..=degree {
        left[j] = t - knot_vector[span + 1 - j];
        right[j] = knot_vector[span + j] - t;

        let mut saved = 0.0;
        let mut temp = 0.0;
        for r in 0..j {
            temp = basis_functions[r] / (right[r + 1] + left[j - r]);
            basis_functions[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        basis_functions[j] = saved;

        basis_derivatives[j] = degree as f64 * (temp - basis_functions[j - 1]) / right[j];
    }

    (basis_functions, basis_derivatives)
}

// Function to compute basis functions
fn basis_functions(t: f64, span: usize, knot_vector: &[f64], degree: usize) -> Vec<f64> {
    // Algorithm to compute basis functions using Cox-de Boor recursion formula
    let mut basis_functions = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    basis_functions[0] = 1.0;

    for j in 1..=degree {
        left[j] = t - knot_vector[span + 1 - j];
        right[j] = knot_vector[span + j] - t;

        let mut saved = 0.0;
        for r in 0..j {
            let temp = basis_functions[r] / (right[r + 1] + left[j - r]);
            basis_functions[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        basis_functions[j] = saved;
    }
    basis_functions
}

// Helper function to compute basis functions and their derivatives
fn basis_functions_derivatives(t: f64, span: usize, knot_vector: &[f64], degree: usize) -> Vec<f64> {
    // Algorithm to compute basis functions and their derivatives using Cox-de Boor recursion formula
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    let mut basis_functions = vec![0.0; degree + 1];
    basis_functions[0] = 1.0;

    for j in 1..=degree {
        left[j] = t - knot_vector[span + 1 - j];
        right[j] = knot_vector[span + j] - t;

        let mut saved = 0.0;
        for r in 0..j {
            let temp = basis_functions[r] / (right[r + 1] + left[j - r]);
            basis_functions[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        basis_functions[j] = saved;
    }
    basis_functions
}



//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////



// Function to compute the derivative of control points for a 3rd order NURBS curve
fn compute_derivative(control_points: &[Vector4<f64>], weights: &[f64], knots: &[f64], order: usize, index: usize) -> Vector4<f64> {
    let n = control_points.len();

    // Get the indices of the control points affected by the derivative calculation
    let start_index = if index >= order - 1 { index - (order - 1) } else { 0 };
    let end_index = if index <= n - order { index } else { n - order + 1 };

    // Initialize the derivative vector
    let mut derivative = Vector4::zeros();

    // Compute the derivative using the weighted sum of neighboring control points
    for i in start_index..=end_index {
        let basis_val = basis_function(i, order, knots);
        derivative += control_points[i] * weights[i] * basis_val;
    }

    // Normalize by the last component (weight)
    derivative /= derivative.w;

    // Subtract the original control point to get the derivative
    derivative - control_points[index]
}

// Function to compute the basis function value at a given knot index
fn basis_function(index: usize, order: usize, knots: &[f64]) -> f64 {
    // Simple piecewise linear basis functions for 3rd order NURBS curve
    if knots[index] < knots[index + order - 1] {
        if knots[index + 1] == knots[index] {
            0.0
        } else {
            (knots[index + 1] - knots[index]) / (knots[index + 1] - knots[index + order - 1])
        }
    } else {
        0.0
    }
}

fn main() {
    // Example parameters
    let control_points = vec![
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 2.0, 0.0, 1.0),
        Vector4::new(3.0, 1.0, 0.0, 1.0),
        Vector4::new(4.0, 3.0, 0.0, 1.0),
    ];
    let weights = vec![1.0, 1.0, 1.0, 1.0];
    let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]; // Example knots for a clamped curve
    let order = 4; // 3rd order NURBS curve

    // Compute the derivative of control points at index 1
    let index = 1;
    let derivative = compute_derivative(&control_points, &weights, &knots, order, index);
    println!("Derivative of control point {}: {:?}", index, derivative);
}



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// 
use nalgebra::{Vector4, Vector3};

// Function to compute the derivative of control points for a NURBS curve
fn compute_derivative(control_points: &[Vector4<f64>], weights: &[f64], knots: &[f64], t: f64) -> Vec<Vector4<f64>> {
    let degree = control_points.len() - knots.len() + 1;

    // Compute the basis functions and their derivatives at parameter t
    let (basis_functions, basis_derivatives) = basis_functions_derivatives(t, find_span(t, knots), knots, degree);

    // Compute the derivative of each control point
    let mut derivative_points = Vec::with_capacity(control_points.len());
    for i in 0..control_points.len() {
        let mut derivative = Vector4::zeros();
        for j in 0..=degree {
            let basis_val = basis_derivatives[j];
            derivative += control_points[i - degree + j] * weights[i - degree + j] * basis_val;
        }
        derivative_points.push(derivative);
    }

    derivative_points
}

// Function to compute the basis functions and their derivatives at parameter t
fn basis_functions_derivatives(t: f64, span: usize, knots: &[f64], degree: usize) -> (Vec<f64>, Vec<f64>) {
    // Initialize the vectors to store basis functions and their derivatives
    let mut basis_functions = vec![0.0; degree + 1];
    let mut basis_derivatives = vec![0.0; degree + 1];

    // Initialize the basis functions with the appropriate values
    basis_functions[0] = 1.0;

    // Initialize the left and right arrays
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];

    // Compute the basis functions and their derivatives using Cox-de Boor recursion formula
    for j in 1..=degree {
        left[j] = t - knots[span + 1 - j];
        right[j] = knots[span + j] - t;

        let mut saved = 0.0;
        for r in 0..j {
            let tmp = basis_functions[r] / (right[r + 1] + left[j - r]);
            basis_functions[r] = saved + right[r + 1] * tmp;
            saved = left[j - r] * tmp;
        }
        basis_functions[j] = saved;
    }

    // Compute the derivatives of the basis functions
    for j in 0..=degree {
        let mut d = vec![0.0; degree + 1];
        d[0] = 1.0;
        let mut a = Vec::with_capacity(degree + 1);
        a.push(d.clone());
        for r in 1..=degree {
            for s in 0..=(degree - r) {
                d[s] = d[s] * (r as f64) / (knots[span + s + 1] - knots[span + s + 1 - r]);
                d[s + 1] = d[s + 1] - d[s];
                a.push(d.clone());
            }
        }
        basis_derivatives[j] = a[j][degree - j];
    }

    (basis_functions, basis_derivatives)
}

// Function to find the span of the given parameter t in the knot vector
fn find_span(t: f64, knots: &[f64]) -> usize {
    let n = knots.len() - 1;
    if t >= knots[n] {
        return n - 1;
    } else if t <= knots[0] {
        return 0;
    }
    let mut low = 0;
    let mut high = n;
    let mut mid = (low + high) / 2;
    while t < knots[mid] || t >= knots[mid + 1] {
        if t < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }
    mid
}

fn main() {
    // Example parameters
    let control_points = vec![
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 2.0, 0.0, 1.0),
        Vector4::new(3.0, 1.0, 0.0, 1.0),
        Vector4::new(4.0, 3.0, 0.0, 1.0),
    ];
    let weights = vec![1.0, 1.0, 1.0, 1.0];
    let knots = vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]; // Example

