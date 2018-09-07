use config::Config;
use shell::Shell;
use ui::{Event, Ui};

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

            match self.shell.poll_output() {
                Ok(Some(string)) => {
                    // append output to output view buffer
                    string_buffer.push_str(&string);
                }
                Ok(None) => {
                    // stream is intact, but has not output
                }
                Err(e) => {
                    return Err(format!("could not read output:\n{}", e));
                }
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
