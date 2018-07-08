use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

pub struct Shell {
    bin_path: PathBuf,
}

impl Shell {
    pub fn new(bin_path: PathBuf) -> Shell {
        return Shell { bin_path: bin_path };
    }

    pub fn execute(&self, command: &str) -> io::Result<Output> {
        return Command::new(&self.bin_path).arg("-c").arg(command).output();
    }
}
