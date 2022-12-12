use std::error::Error;
use std::io::Write;
use tobj;

use crate::panic_log;

pub struct MeshSplitter {
    models: Vec<tobj::Model>,
    mesh_boundry: MeshBoundry,
    mesh_origin: (f32, f32),
    chunk_size: (f32, f32),
    chunks_per_axis: usize,
    chunks: Vec<MeshChunk>,
}

struct MeshBoundry {
    x: (f32, f32),
    z: (f32, f32),
}

struct MeshChunk {
    vertices: Vec<f32>,
    // TODO: faces?
    index_in_mesh: (usize, usize),
    boundries: MeshBoundry,
}

impl MeshSplitter {
    pub fn new(file_path: &str, chunks_per_axis: usize) -> Self {
        log::trace!("MeshSplitter::new({}, {})", file_path, chunks_per_axis);
        log::info!("Start reading input file {}", file_path);

        let models = Self::read_models_from_input_file(file_path);
        log::info!("Reading input file finished");
        if models.len() == 0 {
            panic_log!("Input file does not contain any models!");
        }
        if models.len() > 1 {
            log::warn!("Found {} models in file, using only first", models.len());
        }

        let mesh_boundry = Self::get_mesh_boundry(&models[0].mesh);
        let mesh_origin = (
            mesh_boundry.x.0 + (mesh_boundry.x.1 - mesh_boundry.x.0) / 2.0,
            mesh_boundry.z.0 + (mesh_boundry.z.1 - mesh_boundry.z.0) / 2.0,
        );
        let chunk_size = (
            (mesh_boundry.x.1 - mesh_boundry.x.0) / (chunks_per_axis as f32),
            (mesh_boundry.z.1 - mesh_boundry.z.0) / (chunks_per_axis as f32),
        );

        Self {
            models,
            mesh_boundry,
            mesh_origin,
            chunk_size,
            chunks_per_axis,
            chunks: Vec::with_capacity(chunks_per_axis * chunks_per_axis),
        }
    }

    fn get_mesh(&self) -> &tobj::Mesh {
        &self.models[0].mesh
    }

    fn read_models_from_input_file(file_path: &str) -> Vec<tobj::Model> {
        let obj_file = tobj::load_obj(file_path, &tobj::GPU_LOAD_OPTIONS);
        let (models, _) = match obj_file {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to load OBJ file: {}", file_path);
                panic!("{:?}", e);
            }
        };
        models
    }

    fn get_mesh_boundry(mesh: &tobj::Mesh) -> MeshBoundry {
        let mut min_x = mesh.positions[0];
        let mut max_x = mesh.positions[0];
        let mut min_z = mesh.positions[2];
        let mut max_z = mesh.positions[2];
        for vertex in mesh.positions.chunks(3) {
            if vertex[0] > max_x {
                max_x = vertex[0];
            }
            if vertex[0] < min_x {
                min_x = vertex[0];
            }
            if vertex[2] > max_z {
                max_z = vertex[2];
            }
            if vertex[2] < min_z {
                min_z = vertex[2];
            }
        }
        MeshBoundry { x: (min_x, max_x), z: (min_z, max_z) }
    }

    pub fn run_splitter(&mut self, threads_amount: usize) {
        log::trace!("MeshSplitter::start_splitting({})", threads_amount);
        log::info!(
            "Splitting mesh into {} chunks, chunk size is ({} x {})",
            self.chunks_per_axis * self.chunks_per_axis,
            self.chunk_size.0,
            self.chunk_size.1
        );

        let aprox_vertices_len =
            ((self.get_mesh().positions.len() as f32) / ((self.chunks_per_axis * self.chunks_per_axis) as f32)) * 1.25;
        let aprox_vertices_len = aprox_vertices_len as usize;

        for chunk_i_x in 0..self.chunks_per_axis {
            for chunk_i_z in 0..self.chunks_per_axis {
                let mut new_chunk = MeshChunk {
                    index_in_mesh: (chunk_i_x, chunk_i_z),
                    vertices: Vec::with_capacity(aprox_vertices_len),
                };

                //TODO: fix, seperate function is_vertex_in_chunk or smth
                for vertex in self.get_mesh().positions.chunks(3) {
                    let max_x = (chunk_i_x as f32) * self.chunk_size.0 + self.chunk_size.0;
                    let max_z = (chunk_i_z as f32) * self.chunk_size.1 + self.chunk_size.1;
                    if vertex[0] <= max_x && vertex[2] <= max_z {
                        new_chunk.vertices.push(vertex[0]);
                        new_chunk.vertices.push(vertex[1]);
                        new_chunk.vertices.push(vertex[2]);
                    }
                }

                self.chunks.push(new_chunk);
            }
        }
    }

    pub fn save_chunk_to_file(&self, chunk_pos: (usize, usize)) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_chunk_to_file({:?})", chunk_pos);
        let out_dir_name = "out";

        // create directory and file
        if !std::path::Path::new(out_dir_name).is_dir() {
            std::fs::create_dir(out_dir_name)?;
            log::trace!("Created './{}/' directory", out_dir_name);
        }
        let f = std::fs::File::create(format!("{}/chunk_{}_{}.obj", out_dir_name, chunk_pos.0, chunk_pos.1))?;
        let mut f = std::io::BufWriter::new(f);

        // write all vertex data from original mesh
        for vertex in self.get_mesh().positions.chunks(3) {
            write!(f, "v {:.6} {:.6} {:.6}\n", vertex[0], vertex[1], vertex[2])?;
        }

        // write triangle faces data
        // TODO: fix, take into account mesh start / end
        let min_x = (chunk_pos.0 as f32) * self.chunk_size.0;
        let max_x = min_x + self.chunk_size.0;
        let min_z = (chunk_pos.1 as f32) * self.chunk_size.1;
        let max_z = min_z + self.chunk_size.1;
        for triangle_indices in self.get_mesh().indices.chunks(3) {
            let mut should_run = false;
            let mesh = self.get_mesh();
            for index in triangle_indices {
                let i = (*index as usize) * 3;
                let vertex = (mesh.positions[i], mesh.positions[i + 1], mesh.positions[i + 2]);
                if vertex.0 >= min_x && vertex.0 < max_x && vertex.2 >= min_z && vertex.2 < max_z {
                    should_run = true;
                }
            }
            if should_run {
                write!(f, "f {} {} {}\n", triangle_indices[0], triangle_indices[1], triangle_indices[2])?;
            }
        }

        log::info!("Chunk({}, {}) saved to file", chunk_pos.0, chunk_pos.1);
        Ok(())
    }

    pub fn save_all_chunks_to_file(&self) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_all_chunks_to_file()");
        for chunk in &self.chunks {
            self.save_chunk_to_file(chunk.index_in_mesh)?;
        }
        Ok(())
    }
}