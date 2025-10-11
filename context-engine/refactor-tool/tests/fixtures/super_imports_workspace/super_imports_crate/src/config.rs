#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub debug: bool,
}

impl Config {
    pub fn new(
        name: &str,
        version: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            debug: false,
        }
    }

    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }
}

pub fn load_config() -> Config {
    Config::new("super_imports_test", "0.1.0")
}

pub fn save_config(config: &Config) -> Result<(), String> {
    if config.name.is_empty() {
        Err("Config name cannot be empty".to_string())
    } else {
        Ok(())
    }
}
