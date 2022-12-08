mod logger;

fn main() {
    logger::init();
    let args: Vec<String> = std::env::args().collect();
    let app_config = parallel_viewshed_rust::config::AppConfig::build(&args);

    if let Err(e) = parallel_viewshed_rust::run(app_config) {
        log::error!("Application error: {e}");
        std::process::exit(1);
    }
}