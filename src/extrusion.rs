use super::{Model, DiscreteQuery, vector::*};
use super::mesh::{Mesh, get_trivec_with_offset};
use serde::{Deserialize, Serialize};
use glam::*;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default = "Extrusion::default")]
pub struct Extrusion {
    pub parts:     Vec<Model>,
    pub direction: Box<Model>,
    pub length:    f32,
}

impl Extrusion {
    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh {
        let mat4 = Mat4::from_translation(self.direction.get_vec3_or(Vec3::Z) * self.length);
        let mut vector: Vec<f32> = vec![];
        let mut trivec: Vec<usize> = vec![];
        let mut offset = 0;
        for part in &self.parts {
            let mut polylines = vec![];
            match part {
                Model::Area(area) => {
                    let mesh = area.get_mesh(query);

                    vector.extend(&mesh.vector);
                    let area_trivec: Vec<usize> = mesh.triangles.iter().map(|v| v + offset).collect();
                    trivec.extend(&area_trivec);
                    offset += mesh.vector.len() / 3;

                    let translated_mesh_vec = get_transformed_vector(&mesh.vector, mat4);
                    vector.extend(&translated_mesh_vec);
                    let area_trivec: Vec<usize> = mesh.triangles.iter().map(|v| v + offset).collect();
                    trivec.extend(&area_trivec);
                    offset += mesh.vector.len() / 3;
                    
                    polylines.extend(area.get_polylines(query));
                },
                _ => polylines.extend(part.get_polylines(query)), // polylines.push(part.get_polyline(&query)),
            };
            for polyline in polylines {
                if polyline.len() < 6 { continue }
                let translated_polyline = get_transformed_vector(&polyline, mat4);
                vector.extend(&polyline);
                vector.extend(&translated_polyline);
                trivec.extend(get_trivec_with_offset(2, polyline.len()/3, offset));
                offset += 2 * polyline.len() / 3;
            }
        }
        Mesh {
            vector,
            triangles: trivec,
        }
    }
}

