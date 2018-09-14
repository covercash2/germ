use config::Config;
use shell::Shell;
use ui::{Event, Ui};

use std::time::Instant;

type Error = String;

pub struct App<U: Ui> {
    shell: Shell,
    ui: U,
}

impl<U: Ui> App<U> {
    pub fn new(shell: Shell, ui: U) -> Self {
        return App {
            shell: shell,
            ui: ui,
        };
    }

    fn exit(self) -> Result<(), Error> {
        return Err("not implemented".into());
    }
}
