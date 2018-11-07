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

extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

mod app;
mod config;
mod constants;
mod shell;
mod stream;
mod ui;

use ui::Ui;

use shell::Shell;
use ui::backend::gtk::Gtk;
use ui::Config;

const DEFAULT_DIMENSIONS: [i32; 2] = [600, 600];

fn main() -> Result<(), String> {
    let config: Config = Config::default();

    let mut ui: Gtk =
        Gtk::create("test title".into(), DEFAULT_DIMENSIONS).expect("unable to create gtk app");

    let shell_path = config.shell.path.clone();
    let shell = Shell::create(shell_path.into()).expect("could not create shell");

    return ui.show(shell);
}
