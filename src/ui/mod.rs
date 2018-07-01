pub mod backend;

use std::path::PathBuf;

pub use super::config::Config;
pub use super::constants::{DEFAULT_DIMENSIONS, DEFAULT_TITLE};

pub trait Ui {
    fn init(config: Config) -> Result<Self, String>
    where
        Self: Sized;
    fn show(self) -> Result<(), String>;
}
