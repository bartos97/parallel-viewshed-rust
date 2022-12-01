use tobj;
use std::fs::File;
use std::io::{ BufWriter, Write };

const MODEL_FILE: &str = "resources/blob.obj";

fn main() {
    println!("Start reading file..");
    let obj_file = tobj::load_obj(MODEL_FILE, &tobj::GPU_LOAD_OPTIONS);
    assert!(obj_file.is_ok());

    let (models, _materials) = obj_file.expect("Failed to load OBJ file");
    println!("Found {} models in file", models.len());

    let mesh = &models[0].mesh;
    let triangles_count = mesh.indices.len() / 3;
    println!("Model has {} triangles", triangles_count);

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
    let chunk_max_x = min_x + (max_x - min_x) / 10.0;
    let chunk_max_z = min_z + (max_z - min_z) / 10.0;

    let f = File::create("out_obj.obj").expect("unable to create file");
    let mut f = BufWriter::new(f);

    for i in (0..mesh.positions.len()).step_by(3) {
        write!(f, "v {:.6} {:.6} {:.6}\n", mesh.positions[i], mesh.positions[i + 1], mesh.positions[i + 2]).expect(
            "unable to write"
        );
    }

    // for each triangle
    for i in (0..mesh.indices.len()).step_by(3) {
        let mut should_run = true;
        let triangle_indices: [usize; 3] = [
            mesh.indices[i] as usize,
            mesh.indices[i + 1] as usize,
            mesh.indices[i + 2] as usize,
        ];
        for index in triangle_indices {
            if mesh.positions[index * 3] > chunk_max_x || mesh.positions[index * 3 + 2] > chunk_max_z {
                should_run = false;
            }
        }

        if should_run {
            write!(f, "f {} {} {}\n", triangle_indices[0] + 1, triangle_indices[1] + 1, triangle_indices[2] + 1).expect(
                "unable to write"
            );
        }
    }
}