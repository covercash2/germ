use std::fs::read_to_string;

use toml;
use toml::Value;
use xdg::BaseDirectories;

#[derive(Deserialize)]
pub struct Config {
    pub assets: Assets,
    pub graphics: Graphics,
}

#[derive(Deserialize)]
pub struct Assets {
    pub font_path: String,
}

#[derive(Deserialize)]
pub struct Graphics {
    pub vsync: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        let config_file = BaseDirectories::new()
            .expect("could not read xdg config directory")
            .get_config_home()
            .join("germ/config.toml");
        let config_string: String =
            read_to_string(config_file).expect("could not read config file");

        toml::from_str(&config_string).expect("could not parse config file")
    }
}
