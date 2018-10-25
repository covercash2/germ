#![feature(const_str_as_bytes)]
#![feature(test)]
#![feature(try_trait)]

extern crate test;
#[macro_use]
extern crate conrod;
extern crate font_loader;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate volatile;
extern crate xdg;

mod app;
mod config;
mod constants;
mod shell;
mod stream;
mod ui;

use app::App;

use constants::DEFAULT_FONT;

use shell::Shell;
use ui::backend::conrod::Conrod;
use ui::Config;

fn main() -> Result<(), String> {
    let config: Config = Config::default();
    let ui: Conrod = Conrod::new(
        config.font.family.clone().unwrap_or(DEFAULT_FONT.into()),
        config.graphics.vsync.unwrap_or(false),
    )
    .expect("could not create ui");

    let shell_path = config.shell.path.clone();
    let shell = Shell::create(shell_path.into()).expect("could not create shell");

    let app = App::new(shell, ui);

    return app.run();
}
