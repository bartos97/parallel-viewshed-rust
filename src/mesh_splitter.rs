use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use tobj;

use crate::panic_log;

pub struct MeshSplitter {
    models: Vec<tobj::Model>,
    mesh_boundary: MeshBoundary,
    chunk_size: (f32, f32),
    chunks_per_axis: usize,
    chunks: Vec<MeshChunk>,
}

#[derive(Debug)]
struct MeshBoundary {
    x: (f32, f32),
    z: (f32, f32),
}

struct MeshChunk {
    vertices: Vec<f32>,
    indices: Vec<u32>,
    index_in_mesh: (usize, usize),
    boundary: MeshBoundary,
}

impl MeshSplitter {
    fn get_mesh(&self) -> &tobj::Mesh {
        &self.models[0].mesh
    }

    fn get_chunk_at(&self, chunk_index: (usize, usize)) -> &MeshChunk {
        self.chunks
            .iter()
            .find(|x| x.index_in_mesh == chunk_index)
            .unwrap_or_else(|| {
                panic_log!("Invalid chunk index = {:?}", chunk_index);
            })
    }

    pub fn new(file_path: &str, chunks_per_axis: usize) -> Self {
        log::info!("Reading input file {}...", file_path);

        let models = Self::read_models_from_input_file(file_path);
        log::info!("Reading input file finished");
        if models.len() == 0 {
            panic_log!("Input file does not contain any models!");
        }
        if models.len() > 1 {
            log::warn!("Found {} models in file, using only first", models.len());
        }

        let mesh_boundary = Self::calc_mesh_boundary(&models[0].mesh);
        let mesh_origin = (
            mesh_boundary.x.0 + (mesh_boundary.x.1 - mesh_boundary.x.0) / 2.0,
            mesh_boundary.z.0 + (mesh_boundary.z.1 - mesh_boundary.z.0) / 2.0,
        );
        log::debug!("Mesh origin = {:?}; {:?}", mesh_origin, mesh_boundary);

        let chunk_size = (
            (mesh_boundary.x.1 - mesh_boundary.x.0) / (chunks_per_axis as f32),
            (mesh_boundary.z.1 - mesh_boundary.z.0) / (chunks_per_axis as f32),
        );
        log::debug!("Chunk size = {:?}", chunk_size);

        Self {
            models,
            mesh_boundary,
            chunk_size,
            chunks_per_axis,
            chunks: Vec::with_capacity(chunks_per_axis * chunks_per_axis),
        }
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

    fn calc_mesh_boundary(mesh: &tobj::Mesh) -> MeshBoundary {
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
        MeshBoundary { x: (min_x, max_x), z: (min_z, max_z) }
    }

    pub fn run_splitter(&mut self, threads_amount: usize) {
        log::info!(
            "Splitting mesh into {} chunks using {} threads...",
            self.chunks_per_axis * self.chunks_per_axis,
            threads_amount
        );

        let avg_chunk_vertices_capacity =
            (self.get_mesh().positions.len() as f32) / ((self.chunks_per_axis * self.chunks_per_axis) as f32);
        let avg_chunk_vertices_capacity = avg_chunk_vertices_capacity as usize;
        let avg_chunk_indices_capacity =
            (self.get_mesh().indices.len() as f32) / ((self.chunks_per_axis * self.chunks_per_axis) as f32);
        let avg_chunk_indices_capacity = avg_chunk_indices_capacity as usize;

        for chunk_i_x in 0..self.chunks_per_axis {
            for chunk_i_z in 0..self.chunks_per_axis {
                let mut new_chunk = self.create_chunk(
                    (chunk_i_x, chunk_i_z),
                    avg_chunk_vertices_capacity,
                    avg_chunk_indices_capacity
                );
                // key is the index in original mesh and value is index in new mesh
                let mut vertex_map: HashMap<u32, u32> = HashMap::with_capacity(avg_chunk_indices_capacity);
                let mut vertex_counter: u32 = 1;
                let mesh = self.get_mesh();

                for triangle_indices in mesh.indices.chunks_exact(3) {
                    let mut triangle_vertices: [(f32, f32, f32); 3] = Default::default();
                    for i in 0..triangle_indices.len() {
                        let pos = (triangle_indices[i] as usize) * 3;
                        triangle_vertices[i] = (mesh.positions[pos], mesh.positions[pos + 1], mesh.positions[pos + 2]);
                    }

                    if
                        Self::is_vertex_in_chunk(&triangle_vertices[0], &new_chunk) ||
                        Self::is_vertex_in_chunk(&triangle_vertices[1], &new_chunk) ||
                        Self::is_vertex_in_chunk(&triangle_vertices[2], &new_chunk)
                    {
                        for (vertex, index) in std::iter::zip(triangle_vertices, triangle_indices) {
                            if !vertex_map.contains_key(index) {
                                vertex_map.insert(*index, vertex_counter);
                                vertex_counter += 1;
                                new_chunk.vertices.push(vertex.0);
                                new_chunk.vertices.push(vertex.1);
                                new_chunk.vertices.push(vertex.2);
                            }
                            new_chunk.indices.push(*vertex_map.get(index).unwrap());
                        }
                    }
                }

                self.chunks.push(new_chunk);
            }
        }
        log::info!("Splitting mesh finished");
    }

    fn create_chunk(
        &self,
        chunk_index: (usize, usize),
        vertices_capacity: usize,
        indices_capacity: usize
    ) -> MeshChunk {
        let min_x = self.mesh_boundary.x.0 + (chunk_index.0 as f32) * self.chunk_size.0;
        let max_x = min_x + self.chunk_size.0;
        let min_z = self.mesh_boundary.z.0 + (chunk_index.1 as f32) * self.chunk_size.1;
        let max_z = min_z + self.chunk_size.1;

        MeshChunk {
            index_in_mesh: chunk_index,
            boundary: MeshBoundary {
                x: (min_x, max_x),
                z: (min_z, max_z),
            },
            vertices: Vec::with_capacity(vertices_capacity),
            indices: Vec::with_capacity(indices_capacity),
        }
    }

    fn is_vertex_in_chunk(vertex: &(f32, f32, f32), chunk: &MeshChunk) -> bool {
        vertex.0 >= chunk.boundary.x.0 &&
            vertex.0 <= chunk.boundary.x.1 &&
            vertex.2 >= chunk.boundary.z.0 &&
            vertex.2 <= chunk.boundary.z.1
    }

    pub fn save_all_chunks_to_file(&self) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_all_chunks_to_file()");
        for chunk in &self.chunks {
            self.save_chunk_to_file(chunk)?;
        }
        Ok(())
    }

    pub fn save_chunk_at_to_file(&self, chunk_index: (usize, usize)) -> Result<(), Box<dyn Error>> {
        self.save_chunk_to_file(self.get_chunk_at(chunk_index))
    }

    fn save_chunk_to_file(&self, chunk: &MeshChunk) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_chunk_to_file({:?})", chunk.index_in_mesh);
        let out_dir_name = "out";

        // create directory and file
        if !std::path::Path::new(out_dir_name).is_dir() {
            std::fs::create_dir(out_dir_name)?;
            log::trace!("Created './{}/' directory", out_dir_name);
        }
        let f = std::fs::File::create(
            format!("{}/chunk_{}_{}.obj", out_dir_name, chunk.index_in_mesh.0, chunk.index_in_mesh.1)
        )?;
        let mut f = std::io::BufWriter::new(f);

        for vertex in chunk.vertices.chunks_exact(3) {
            write!(f, "v {:.6} {:.6} {:.6}\n", vertex[0], vertex[1], vertex[2])?;
        }

        for triangle_indices in chunk.indices.chunks_exact(3) {
            write!(f, "f {} {} {}\n", triangle_indices[0], triangle_indices[1], triangle_indices[2])?;
        }

        log::info!("Chunk({}, {}) saved to file", chunk.index_in_mesh.0, chunk.index_in_mesh.1);
        Ok(())
    }
}