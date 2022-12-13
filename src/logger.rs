use std::{ io::Write };

pub fn init() {
    use simplelog::*;

    if !std::path::Path::new("logs/").is_dir() {
        std::fs::create_dir("logs").unwrap();
    }

    let log_file_config = ConfigBuilder::new().set_time_format_rfc3339().build();
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H%M%SZ").to_string();

    if cfg!(debug_assertions) {
        let log_file_path = String::from("logs/debug.log");
        let mut log_file = std::fs::OpenOptions
            ::new()
            .write(true)
            .create(true)
            .append(true)
            .open(log_file_path)
            .unwrap();
        writeln!(log_file, "--\nStarted at {}\n--", timestamp).expect("Unabled to write to debug log file");

        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Trace, log_file_config, log_file)
            ]
        ).unwrap();
    } else {
        let log_file_path = format!("logs/{}.log", timestamp);
        let log_file = std::fs::File::create(log_file_path).unwrap();

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
    ($($arg:tt)+) => {
        log::error!($($arg)+);
        panic!($($arg)+);
    };
}