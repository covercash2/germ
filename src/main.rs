#[macro_use]
extern crate conrod;
extern crate font_loader;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xdg;

mod app;
mod config;
mod constants;
mod shell;
mod ui;

use constants::DEFAULT_FONT;

use shell::Shell;
use ui::backend::gtk::Gtk;
use ui::{Config, Ui};

fn main() -> Result<(), String> {
    let config: Config = Config::default();
    // let ui: Conrod = Conrod::new(
    //     config.font.family.clone().unwrap_or(DEFAULT_FONT.into()),
    //     config.graphics.vsync.unwrap_or(false),
    // ).expect("could not create ui");

    let ui: Gtk = Gtk::create("testing".into(), [600, 600]).expect("could not create ui");

    let shell_path = config.shell.path.clone();
    let shell = Shell::create(shell_path.into());

    return ui.run(shell);
}
