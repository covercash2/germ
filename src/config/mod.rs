use std::fs::read_to_string;
use std::path::Path;

use toml;
use xdg::BaseDirectories;

#[derive(Deserialize)]
pub struct Config {
    pub font: Font,
    pub graphics: Graphics,
}

#[derive(Deserialize)]
pub struct Font {
    pub family: Option<String>,
    pub size: Option<i64>,
}

#[derive(Deserialize)]
pub struct Graphics {
    pub vsync: Option<bool>,
}

impl Config {
    fn load<P: AsRef<Path>>(config_file: P) -> Result<Config, String> {
        match read_to_string(&config_file) {
            Ok(string) => {
                toml::from_str(&string).map_err(|e| format!("could not parse toml config:\n{}", e))
            }
            Err(io_err) => Err(format!(
                "could not read config file: {:?}\n{}",
                config_file.as_ref(),
                io_err
            )),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let config_file = BaseDirectories::new()
            .expect("could not read xdg config directory")
            .get_config_home()
            .join("germ/config.toml");

        return match Config::load(config_file) {
            Ok(config) => config,
            Err(e) => panic!(format!("could not load default config:\n{}", e)),
        };
    }
}
