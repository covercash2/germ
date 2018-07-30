use config::Config;
use shell::Shell;
use ui::{Event, Ui};

use std::io::{BufReader, Read};
use std::sync::mpsc;
use std::thread;

const DEFAULT_OUTPUT_BUFFER_SIZE: usize = 10;

type Error = String;
type OutputBuffer = [u8; DEFAULT_OUTPUT_BUFFER_SIZE];

pub struct App<U: Ui> {
    config: Config,
    ui: U,
}

impl<U: Ui> App<U> {
    pub fn new(config: Config, ui: U) -> Self {
        return App {
            config: config,
            ui: ui,
        };
    }

    pub fn run(mut self) -> Result<(), String> {
        let mut output = String::new();

        let shell_path = self.config.shell.path.clone();
        let mut shell = Shell::new(shell_path.into());

        let mut string_buffer = String::new();

        shell.execute("ls\n").unwrap();

        'main: loop {
            for event in self.ui.events() {
                match event {
                    Event::Submit(command) => {
                        eprintln!("submitted: {:?}", command);

                        // TODO sanitize commands
                        let mut command = String::from(command);
                        if !command.ends_with('\n') {
                            command.push('\n');
                        }

                        shell.execute(&command).expect("could not execute command");

                        // TODO clear output view
                    }
                    // break loop
                    Event::Exit => return Ok(()),
                }
            }

            match shell.poll_output() {
                Some(line) => {
                    string_buffer.push_str(&line);
                    // TODO ugly
                    string_buffer.push('\n');
                }
                None => (),
            }

            self.ui.set_output(&string_buffer);

            match self.ui.draw() {
                Ok(()) => (),
                Err(e) => return Err(format!("error drawing ui:\n{}", e)),
            }
        }
    }

    fn exit(self) -> Result<(), Error> {
        return Err("not implemented".into());
    }
}
