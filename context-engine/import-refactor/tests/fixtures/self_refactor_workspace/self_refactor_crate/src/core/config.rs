#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub debug_mode: bool,
}

impl Config {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            debug_mode: false,
        }
    }
    
    pub fn with_debug(mut self) -> Self {
        self.debug_mode = true;
        self
    }
}

pub fn load_settings() -> Config {
    Config::new("MyApp")
}

pub fn save_settings(config: &Config) -> Result<(), String> {
    println!("Saving config: {:?}", config);
    Ok(())
}