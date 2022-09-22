use tobj;
use std::fs::File;
use std::io::{ BufWriter, Write };

const MODEL_FILE: &str = "resources/teapot.obj";

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
    for pos in (0..mesh.positions.len()).step_by(3) {
        let x = mesh.positions[pos];
        // let y = mesh.positions[pos + 1];
        // let z = mesh.positions[pos + 2];
        if x > max_x {
            max_x = x;
        }
        if x < min_x {
            min_x = x;
        }
    }
    let half_x = min_x + (max_x - min_x) / 2.0;

    //print vertices from half of model in X axis
    // for pos in (0..mesh.positions.len()).step_by(3) {
    //     let x = mesh.positions[pos];
    //     let y = mesh.positions[pos + 1];
    //     let z = mesh.positions[pos + 2];
    //     if x < half_x {
    //         println!("x = {}, y = {}, z = {}", x, y, z);
    //     }
    // }

    let f = File::create("out_faces.txt").expect("unable to create file");
    let mut f = BufWriter::new(f);

    // for each triangle
    for i in (0..mesh.indices.len()).step_by(3) {
        let mut should_run = true;
        let triangle_indices: [usize; 3] = [
            mesh.indices[i] as usize,
            mesh.indices[i + 1] as usize,
            mesh.indices[i + 2] as usize,
        ];
        for index in triangle_indices {
            // vertex = [x, y, z]
            let vertex = [mesh.positions[index * 3], mesh.positions[index * 3 + 1], mesh.positions[index * 3 + 2]];
            if vertex[0] > half_x {
                should_run = false;
            }
        }

        if should_run {
            write!(f, "f {} {} {}\n", triangle_indices[0], triangle_indices[1], triangle_indices[2]).expect(
                "unable to write"
            );
        }
    }
}