#[derive(Debug)]
pub struct AppConfig {
    pub file_path: String,
    pub chunks_per_axis: usize,
}

static DEFAULT_FILE_PATH: &str = "input.obj";
static DEFAULT_CHUNKS_PER_AXIS: usize = 10;

impl AppConfig {
    pub fn build(mut args: impl Iterator<Item = String>) -> AppConfig {
        args.next(); //skip program name

        let file_path = match args.next() {
            Some(arg) => arg,
            None => {
                log::warn!("No input file path passed as program argument - using default = {}", DEFAULT_FILE_PATH);
                String::from(DEFAULT_FILE_PATH)
            }
        };

        let chunks_per_axis = match args.next() {
            Some(arg) => {
                match arg.parse::<usize>() {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        log::warn!(
                            "Unable to parse chunks per axis value (\"{}\") passed as program argument - using default = {}",
                            arg,
                            DEFAULT_CHUNKS_PER_AXIS
                        );
                        DEFAULT_CHUNKS_PER_AXIS
                    }
                }
            }
            None => {
                log::warn!(
                    "No chunks per axis value passed as program argument - using default = {}",
                    DEFAULT_CHUNKS_PER_AXIS
                );
                DEFAULT_CHUNKS_PER_AXIS
            }
        };

        let config = Self {
            file_path,
            chunks_per_axis,
        };
        log::debug!("{:#?}", config);
        config
    }
}