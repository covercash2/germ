use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};

pub struct Shell {
    bin_path: PathBuf,
    process: Child,
}

impl Shell {
    pub fn new(bin_path: PathBuf) -> Shell {
        let child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("should work dummy");
        return Shell {
            bin_path: bin_path,
            process: child,
        };
    }

    pub fn execute(&mut self, command: &str) -> io::Result<Output> {
        match self.process.stdin {
            Some(ref mut stdin) => {
                stdin
                    .write_all(command.as_ref())
                    .expect("could not write to subprocess stdin");
            }
            None => eprintln!("couldn't get stdin"),
        }
        return Command::new(&self.bin_path).arg("-c").arg(command).output();
    }

    pub fn copy_output(&mut self, buf: &mut String) -> io::Result<usize> {
        match self.process.stdout {
            Some(ref mut stdout) => {
                return stdout.read_to_string(buf);
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not get stdout of child process",
                ));
            }
        }
    }
}
