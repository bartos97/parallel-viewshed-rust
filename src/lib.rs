pub mod config;

use std::io::Write;
use std::error::Error;

pub fn run(app_config: config::AppConfig) -> Result<(), Box<dyn Error>> {
    let models = read_models_from_input_file(&app_config.file_path);
    if models.len() > 1 {
        log::warn!("Found {} models in file, using only first", models.len());
    }
    let mesh = &models[0].mesh;
    log::info!("Model has {} triangles", mesh.indices.len() / 3);

    let (chunk_size_x, chunk_size_z) = get_chunk_max_size(app_config.chunks_per_axis, mesh);

    let f = std::fs::File::create("out_obj.obj")?;
    let mut f = std::io::BufWriter::new(f);

    for i in (0..mesh.positions.len()).step_by(3) {
        let vertex = (mesh.positions[i], mesh.positions[i + 1], mesh.positions[i + 2]);
        write!(f, "v {:.6} {:.6} {:.6}\n", vertex.0, vertex.1, vertex.2)?;
    }
    log::trace!("Finished writing vertices to file");

    // for each triangle
    for i in (0..mesh.indices.len()).step_by(3) {
        let mut should_run = true;
        let triangle_indices = [mesh.indices[i] as usize, mesh.indices[i + 1] as usize, mesh.indices[i + 2] as usize];
        for index in triangle_indices {
            if mesh.positions[index * 3] > chunk_size_x || mesh.positions[index * 3 + 2] > chunk_size_z {
                should_run = false;
            }
        }

        if should_run {
            write!(f, "f {} {} {}\n", triangle_indices[0] + 1, triangle_indices[1] + 1, triangle_indices[2] + 1)?;
        }
    }
    log::trace!("Finished writing faces to file");

    Ok(())
}

fn get_chunk_max_size(chunks_per_axis: u32, mesh: &tobj::Mesh) -> (f32, f32) {
    log::info!("Splitting model into {} chunks", chunks_per_axis);

    let mut min_x = mesh.positions[0];
    let mut max_x = mesh.positions[0];
    let mut min_z = mesh.positions[1];
    let mut max_z = mesh.positions[1];
    for pos in (0..mesh.positions.len()).step_by(3) {
        let x = mesh.positions[pos];
        if x > max_x {
            max_x = x;
        }
        if x < min_x {
            min_x = x;
        }
        let z = mesh.positions[pos + 2];
        if z > max_z {
            max_z = z;
        }
        if z < min_z {
            min_z = z;
        }
    }

    let chunk_size_x = min_x + (max_x - min_x) / (chunks_per_axis as f32);
    let chunk_size_z = min_z + (max_z - min_z) / (chunks_per_axis as f32);
    (chunk_size_x, chunk_size_z)
}

fn read_models_from_input_file(file_path: &str) -> Vec<tobj::Model> {
    log::info!("Start reading file {}", file_path);
    let obj_file = tobj::load_obj(file_path, &tobj::GPU_LOAD_OPTIONS);
    let (models, _) = match obj_file {
        Ok(res) => res,
        Err(e) => {
            log::error!("Failed to load OBJ file");
            panic!("{:?}", e);
        }
    };
    models
}