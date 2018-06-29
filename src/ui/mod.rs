pub mod backend;

use std::path::PathBuf;

use super::constants::{DEFAULT_DIMENSIONS, DEFAULT_TITLE};

pub struct Config {
    assets_folder: PathBuf,
    dimensions: [u32; 2],
    font_path: PathBuf,
    title: String,
    vsync: bool,
}

impl Default for Config {
    fn default() -> Self {
        let assets_folder = PathBuf::from("./assets")
            .canonicalize()
            .expect("could not find assets folder");
        let font_path = assets_folder.join("./fonts/Default.ttf");

        return Config {
            assets_folder: assets_folder,
            dimensions: DEFAULT_DIMENSIONS,
            font_path: font_path,
            title: DEFAULT_TITLE.into(),
            vsync: false,
        };
    }
}

pub trait Ui {
    fn init(config: Config) -> Result<Self, String>
    where
        Self: Sized;
    fn show(self) -> Result<(), String>;
}
