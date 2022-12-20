pub mod config;
pub mod logger;
pub mod mesh_splitter;

use std::error::Error;
use crate::mesh_splitter::MeshSplitter;

pub fn run(app_config: config::AppConfig) -> Result<(), Box<dyn Error>> {
    let mut splitter = MeshSplitter::new(&app_config.file_path, app_config.chunks_per_axis);
    splitter.run_splitter();
    splitter.save_chunk_at_to_file((0, 0))?;

    Ok(())
}