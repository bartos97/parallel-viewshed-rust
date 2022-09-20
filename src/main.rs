use tobj;

fn main() {
    let obj_file = tobj::load_obj("resources/teapot.obj", &tobj::GPU_LOAD_OPTIONS);
    assert!(obj_file.is_ok());
    let (models, _materials) = obj_file.expect("Failed to load OBJ file");
    println!("# of models: {}", models.len());
}