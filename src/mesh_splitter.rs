use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use tobj;

use crate::panic_log;

const OUT_DIR_NAME: &str = "out";

#[derive(Default)]
struct Vertex(f32, f32, f32);

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

impl MeshChunk {
    fn new(
        chunk_index: (usize, usize),
        chunk_size: (f32, f32),
        mesh_boundary: &MeshBoundary,
        vertices_capacity: usize,
        indices_capacity: usize
    ) -> Self {
        let min_x = mesh_boundary.x.0 + (chunk_index.0 as f32) * chunk_size.0;
        let max_x = min_x + chunk_size.0;
        let min_z = mesh_boundary.z.0 + (chunk_index.1 as f32) * chunk_size.1;
        let max_z = min_z + chunk_size.1;

        Self {
            index_in_mesh: chunk_index,
            boundary: MeshBoundary {
                x: (min_x, max_x),
                z: (min_z, max_z),
            },
            vertices: Vec::with_capacity(vertices_capacity),
            indices: Vec::with_capacity(indices_capacity),
        }
    }

    fn is_vertex_inside(&self, vertex: &Vertex) -> bool {
        vertex.0 >= self.boundary.x.0 &&
            vertex.0 <= self.boundary.x.1 &&
            vertex.2 >= self.boundary.z.0 &&
            vertex.2 <= self.boundary.z.1
    }

    fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_chunk_to_file({:?})", self.index_in_mesh);

        // create directory and file
        if !std::path::Path::new(OUT_DIR_NAME).is_dir() {
            std::fs::create_dir(OUT_DIR_NAME)?;
            log::trace!("Created './{}/' directory", OUT_DIR_NAME);
        }
        let f = std::fs::File::create(
            format!("{}/chunk_{}_{}.obj", OUT_DIR_NAME, self.index_in_mesh.0, self.index_in_mesh.1)
        )?;
        let mut f = std::io::BufWriter::new(f);

        for vertex in self.vertices.chunks_exact(3) {
            write!(f, "v {:.6} {:.6} {:.6}\n", vertex[0], vertex[1], vertex[2])?;
        }

        for triangle_indices in self.indices.chunks_exact(3) {
            write!(f, "f {} {} {}\n", triangle_indices[0], triangle_indices[1], triangle_indices[2])?;
        }

        log::info!("Chunk({}, {}) saved to file", self.index_in_mesh.0, self.index_in_mesh.1);
        Ok(())
    }
}

#[allow(unused)]
pub struct MeshSplitter {
    models: Vec<tobj::Model>,
    mesh_boundary: MeshBoundary,
    chunk_size: (f32, f32),
    chunks_per_axis: usize,
    chunks: Vec<MeshChunk>,
}

impl MeshSplitter {
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

        let chunks = Self::setup_empty_chunks(&models[0].mesh, chunk_size, chunks_per_axis, &mesh_boundary);

        Self {
            models,
            mesh_boundary,
            chunk_size,
            chunks_per_axis,
            chunks,
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

    fn setup_empty_chunks(
        mesh: &tobj::Mesh,
        chunk_size: (f32, f32),
        chunks_per_axis: usize,
        mesh_boundary: &MeshBoundary
    ) -> Vec<MeshChunk> {
        let avg_chunk_vertices_capacity = (mesh.positions.len() as f32) / ((chunks_per_axis * chunks_per_axis) as f32);
        let avg_chunk_vertices_capacity = avg_chunk_vertices_capacity as usize;
        let avg_chunk_indices_capacity = (mesh.indices.len() as f32) / ((chunks_per_axis * chunks_per_axis) as f32);
        let avg_chunk_indices_capacity = avg_chunk_indices_capacity as usize;

        let mut chunks: Vec<MeshChunk> = Vec::with_capacity(chunks_per_axis * chunks_per_axis);
        for chunk_i_x in 0..chunks_per_axis {
            for chunk_i_z in 0..chunks_per_axis {
                chunks.push(
                    MeshChunk::new(
                        (chunk_i_x, chunk_i_z),
                        chunk_size,
                        mesh_boundary,
                        avg_chunk_vertices_capacity,
                        avg_chunk_indices_capacity
                    )
                );
            }
        }
        chunks
    }

    pub fn run_splitter(&mut self) {
        log::info!("Splitting mesh into {} chunks", self.chunks_per_axis * self.chunks_per_axis);
        
        self.chunks.par_iter_mut().for_each(|chunk| {
            // key is the index in original mesh and value is index in new mesh
            let mut vertex_map: HashMap<u32, u32> = HashMap::with_capacity(chunk.indices.capacity());
            let mut vertex_counter: u32 = 1;
            let mesh = &self.models[0].mesh;
    
            for triangle_indices in mesh.indices.chunks_exact(3) {
                let mut triangle_vertices: [Vertex; 3] = Default::default();
                for i in 0..triangle_indices.len() {
                    let pos = (triangle_indices[i] as usize) * 3;
                    triangle_vertices[i] = Vertex(
                        mesh.positions[pos],
                        mesh.positions[pos + 1],
                        mesh.positions[pos + 2]
                    );
                }
    
                if
                    chunk.is_vertex_inside(&triangle_vertices[0]) ||
                    chunk.is_vertex_inside(&triangle_vertices[1]) ||
                    chunk.is_vertex_inside(&triangle_vertices[2])
                {
                    for (vertex, index) in std::iter::zip(triangle_vertices, triangle_indices) {
                        if !vertex_map.contains_key(index) {
                            vertex_map.insert(*index, vertex_counter);
                            vertex_counter += 1;
                            chunk.vertices.push(vertex.0);
                            chunk.vertices.push(vertex.1);
                            chunk.vertices.push(vertex.2);
                        }
                        chunk.indices.push(*vertex_map.get(index).unwrap());
                    }
                }
            }
        });

        log::info!("Splitting mesh finished");
    }

    pub fn save_all_chunks_to_file(&self) -> Result<(), Box<dyn Error>> {
        log::trace!("MeshSplitter::save_all_chunks_to_file()");
        for chunk in &self.chunks {
            chunk.save_to_file()?;
        }
        Ok(())
    }

    pub fn save_chunk_at_to_file(&self, chunk_index: (usize, usize)) -> Result<(), Box<dyn Error>> {
        self.get_chunk_at(chunk_index).save_to_file()?;
        Ok(())
    }

    fn get_chunk_at(&self, chunk_index: (usize, usize)) -> &MeshChunk {
        self.chunks
            .iter()
            .find(|x| x.index_in_mesh == chunk_index)
            .unwrap_or_else(|| {
                panic_log!("Invalid chunk index = {:?}", chunk_index);
            })
    }
}