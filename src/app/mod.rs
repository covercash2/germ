use config::Config;
use shell::Shell;
use ui::{Event, Ui};

use std::sync::mpsc;

pub struct App<U: Ui> {
    config: Config,
    shell: Shell,
    ui: U,
}

impl<U: Ui> App<U> {
    pub fn new(config: Config, shell: Shell, ui: U) -> Self {
        return App {
            config: config,
            shell: shell,
            ui: ui,
        };
    }

    pub fn run(mut self) -> Result<(), String> {
        let mut output = String::new();

        'main: loop {
            for event in self.ui.events() {
                match event {
                    Event::Submit(command) => {
                        eprintln!("submitted: {}", command);
                        // TODO should not block
                        match self.shell.execute(&command) {
                            Ok(output) => {
                                let string = String::from_utf8(output.stdout).unwrap();
                                self.ui.set_output(&string);
                            }
                            Err(e) => {
                                return Err(format!(
                                    "error executing command \"{}\":\n{}",
                                    command, e
                                ))
                            }
                        }
                    }
                    // break loop
                    Event::Exit => return Ok(()),
                }
            }

            // TODO async?
            // self.shell
            //     .copy_output(&mut output)
            //     .expect("could not copy output to buffer");

            // self.shell
            //     .copy_output(&mut output_buffer)
            //     .expect("could not copy output to buffer");

            match self.ui.draw() {
                Ok(()) => (),
                Err(e) => return Err(format!("error drawing ui:\n{}", e)),
            }
        }
    }
}
