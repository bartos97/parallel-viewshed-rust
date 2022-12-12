use parallel_viewshed_rust::{config::AppConfig, logger};

fn main() {
    logger::init();
    let args: Vec<String> = std::env::args().collect();
    let app_config = AppConfig::build(&args);

    if let Err(e) = parallel_viewshed_rust::run(app_config) {
        log::error!("Application error: {e}");
        std::process::exit(1);
    }
}