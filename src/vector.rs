use glam::*;

pub fn get_transformed_vector(vector: &Vec<f32>, matrix: Mat4) -> Vec<f32> {
    let mut result = vec![];
    for v in vector.chunks(3) {
        let vec4 = Vec4::new(v[0], v[1], v[2], 1.); //Vec4::from_slice(v);
        let array = matrix.mul_vec4(vec4).to_array();
        result.extend([array[0], array[1], array[2]]);
    }
    result
}