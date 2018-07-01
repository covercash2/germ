#[macro_use]
extern crate conrod;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xdg;

mod config;
mod constants;
mod ui;

use ui::backend::conrod::Conrod;
use ui::{Config, Ui};

fn main() -> Result<(), String> {
    let ui: Conrod = match Conrod::init(Config::default()) {
        Ok(u) => u,
        Err(e) => {
            return Err(format!("could not create ui: {}", e));
        }
    };

    return ui.show();
}
