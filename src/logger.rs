pub fn init() {
    use simplelog::*;

    if !std::path::Path::new("logs/").is_dir() {
        std::fs::create_dir("logs").unwrap();
    }
    let log_file_config = ConfigBuilder::new().set_time_format_rfc3339().build();
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H%M%SZ").to_string();
    let log_file_path = if cfg!(debug_assertions) {
        format!("logs/debug_{}.log", timestamp)
    } else {
        format!("logs/{}.log", timestamp)
    };
    let log_file = std::fs::File::create(log_file_path).unwrap();

    if cfg!(debug_assertions) {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Trace, log_file_config, log_file)
            ]
        ).unwrap();
    } else {
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Trace, log_file_config, log_file)
            ]
        ).unwrap();
    }
}

#[macro_export]
macro_rules! panic_log {
    ($msg:tt) => {
        log::error!($msg);
        panic!($msg);
    };
}