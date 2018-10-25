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

    pub fn run(mut self) -> Result<(), Error> {
        let mut string_buffer = String::new();

        'main: loop {
            let frame_start = Instant::now();
            for event in self.ui.events() {
                match event {
                    Event::Submit(command) => {
                        eprintln!("submitted: {:?}", command);

                        // TODO sanitize commands
                        let mut command = String::from(command);
                        if !command.ends_with('\n') {
                            command.push('\n');
                        }

                        self.shell
                            .execute(&command)
                            .expect("could not execute command");

                        string_buffer.clear();
                    }
                    // break loop
                    Event::Exit => return Ok(()),
                }
            }

            match self.shell.poll_stdout() {
                Ok(Some(vec)) => {
                    string_buffer.push_str(
                        ::std::str::from_utf8(&vec).expect("could not push string to buffer"),
                    );
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(format!("could not read output:\n{}", e));
                }
            }

            self.ui.set_output(&string_buffer);

            match self.ui.draw() {
                Ok(()) => (),
                Err(e) => return Err(format!("error drawing ui:\n{}", e)),
            }

            let fps = 1_000_000_000.0 / frame_start.elapsed().subsec_nanos() as f64;
            if fps < 20.0 {
                eprintln!("fps: {}", fps);
            }
        }
    }

    fn exit(self) -> Result<(), Error> {
        return Err("not implemented".into());
    }
}
