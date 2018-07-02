#[macro_use]
extern crate conrod;
extern crate font_loader;
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
    let ui: Conrod = Conrod::init(Config::default()).expect("could not create ui");

    return ui.show();
}
