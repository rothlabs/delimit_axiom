use web_sys::WebGlProgram;

use crate::{gpu::GPU, hit::shaders2::{INIT_PALETTE_SOURCE, HONE_SOURCE, SCORE_SOURCE}};

pub struct Program {
    pub initial: WebGlProgram,
    pub palette: WebGlProgram,
    pub score:   WebGlProgram,
}

impl Program {
    pub fn new(gpu: &GPU) -> Vec<Vec<Program>> {
        vec![
            vec![],
            vec![
                Program {
                    initial: gpu.quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
                    palette: gpu.quad_program_from_source(HONE_SOURCE).unwrap(),
                    score:   gpu.quad_program_from_source(SCORE_SOURCE).unwrap(),
                },
                Program {
                    initial: gpu.quad_program_from_source(INIT_PALETTE_SOURCE).unwrap(),
                    palette: gpu.quad_program_from_source(HONE_SOURCE).unwrap(),
                    score:   gpu.quad_program_from_source(SCORE_SOURCE).unwrap(),
                }
            ],
        ]
    } 
}