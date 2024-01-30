use super::{Model, DiscreteQuery};
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
    pub fn get_mesh(&self, query: &DiscreteQuery) -> Mesh {
        //let MeshQuery {count, tolerance, ..} = query;
        let mut vector: Vec<f32> = vec![];
        let mut trivec: Vec<usize> = vec![];
        let mut offset = 0;
        for part in &self.parts {
            //let mut polylines:Vec<Vec<f32>> = vec![];
            let polylines = match part {
                Model::Area(area) => {
                    let mesh = area.get_mesh(query.tolerance);
                    let translated_mesh_vec = get_translated_vector(&mesh.vector, self.direction, self.distance);
                    vector.extend(&translated_mesh_vec);
                    let area_trivec: Vec<usize> = mesh.triangles.iter().map(|v| v + offset).collect();
                    trivec.extend(&area_trivec);
                    offset += mesh.vector.len() / 3;
                    area.get_polylines(query)
                },
                _ => vec![part.get_polyline(&query)],
            };
            for polyline in polylines {
                if polyline.len() < 6 { continue }
                let translated_polyline = get_translated_vector(&polyline, self.direction, self.distance);
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

fn get_translated_vector(vector: &Vec<f32>, dir: [f32; 3], dist: f32) -> Vec<f32> {
    vector.chunks(3).map(|v| [
        v[0] + dir[0] * dist, 
        v[1] + dir[1] * dist, 
        v[2] + dir[2] * dist,
    ]).flatten().collect()
}


                // Model::Nurbs(m)     => vec![m.get_polyline(*count)],
                // Model::Path(m)      => vec![m.get_polyline(*tolerance)],
                // Model::Circle(m)    => vec![m.get_polyline(*tolerance)],
                // Model::Rectangle(m) => vec![m.get_polyline(*tolerance)],

                // let moved_polyline: Vec<f32> = polyline.chunks(3).map(|v| [
                //     v[0] + self.direction[0] * self.distance, 
                //     v[1] + self.direction[1] * self.distance, 
                //     v[2] + self.direction[2] * self.distance,
                // ]).flatten().collect();