pub struct AppConfig {
    pub file_path: String,
    pub chunks_per_axis: usize,
    pub threads_amount: usize,
}

static DEFAULT_FILE_PATH: &str = "input.obj";
static DEFAULT_CHUNKS_PER_AXIS: usize = 10;
static DEFAULT_THREADS_AMOUNT: usize = 6;

impl AppConfig {
    pub fn build(args: &[String]) -> AppConfig {
        if args.len() <= 1 {
            log::warn!("No input file path passed as program argument - using default = {}", DEFAULT_FILE_PATH);
        }
        if args.len() <= 2 {
            log::warn!(
                "No chunks per axis value passed as program argument - using default = {}",
                DEFAULT_CHUNKS_PER_AXIS
            );
        }
        if args.len() <= 3 {
            log::warn!(
                "No threads amount value passed as program argument - using default = {}",
                DEFAULT_THREADS_AMOUNT
            );
        }

        Self {
            file_path: if args.len() > 1 {
                args[1].clone()
            } else {
                String::from(DEFAULT_FILE_PATH)
            },

            chunks_per_axis: if args.len() > 2 {
                match args[2].parse::<usize>() {
                    Ok(x) => x,
                    Err(_) => {
                        log::warn!(
                            "Unable to parse chunks per axis value (\"{}\") passed as program argument - using default = {}",
                            args[2],
                            DEFAULT_CHUNKS_PER_AXIS
                        );
                        DEFAULT_CHUNKS_PER_AXIS
                    }
                }
            } else {
                DEFAULT_CHUNKS_PER_AXIS
            },

            threads_amount: if args.len() > 3 {
                match args[3].parse::<usize>() {
                    Ok(x) => x,
                    Err(_) => {
                        log::warn!(
                            "Unable to parse threads amount value (\"{}\") passed as program argument - using default = {}",
                            args[3],
                            DEFAULT_THREADS_AMOUNT
                        );
                        DEFAULT_THREADS_AMOUNT
                    }
                }
            } else {
                DEFAULT_THREADS_AMOUNT
            },
        }
    }
}