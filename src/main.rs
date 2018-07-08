#[macro_use]
extern crate conrod;
extern crate font_loader;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xdg;

mod app;
mod config;
mod constants;
mod shell;
mod ui;

use app::App;

use constants::DEFAULT_FONT;

use shell::Shell;

use ui::backend::conrod::Conrod;
use ui::Config;

fn main() -> Result<(), String> {
    let config: Config = Config::default();
    let shell: Shell = Shell::new(config.shell.path.clone().into());
    let ui: Conrod = Conrod::new(
        config.font.family.clone().unwrap_or(DEFAULT_FONT.into()),
        config.graphics.vsync.unwrap_or(false),
    ).expect("could not create ui");

    let app = App::new(config, shell, ui);

    return app.run();
}
