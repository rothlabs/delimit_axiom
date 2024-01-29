use super::Model;
use super::mesh::{Mesh, get_trivec_with_offset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Extrusion::default")]
pub struct Extrusion {
    pub parts:     Vec<Model>,
    pub direction: [f32; 3],
    pub distance:  f32,
}

impl Extrusion {
    pub fn get_mesh(&self, tolerance: f32) -> Mesh {
        let mut vector: Vec<f32> = vec![];
        let mut triangles: Vec<usize> = vec![];
        let mut offset = 0;
        for part in &self.parts {
            let polylines = match part {
                Model::Path2D(path_2d) => vec![path_2d.get_polyline(tolerance)],
                Model::Area(area)      => area.get_polylines(tolerance),
                _ => vec![],
            };
            for polyline in polylines {
                if polyline.len() < 6 { continue }
                vector.extend(polyline.clone());
                let moved_polyline: Vec<f32> = polyline.chunks(3).map(|v| [
                    v[0] + self.direction[0] * self.distance, 
                    v[1] + self.direction[1] * self.distance, 
                    v[2] + self.direction[2] * self.distance,
                ]).flatten().collect();
                vector.extend(moved_polyline);
                triangles.extend(get_trivec_with_offset(2, polyline.len()/3, offset));
                offset += 2 * polyline.len() / 3;
            }
        }
        Mesh {
            vector,
            triangles,
        }
    }
}