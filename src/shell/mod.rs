use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::iter::Take;
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender, TryIter, TryRecvError};
use std::thread::spawn;
use std::thread::JoinHandle;

mod stream;

pub trait AsyncShell {
    type Buffer;
    type Error;
    fn send(&mut self, command: &str) -> Result<(), Self::Error>;
    fn receive(&self) -> Result<Self::Buffer, Self::Error>;
}

pub struct Shell {
    stdin: ChildStdin,
    stdout: stream::StringStream,
    stderr: stream::StringStream,
}

impl Shell {
    pub fn create(bin_path: PathBuf) -> Shell {
        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("should work dummy");

        let stdin = child.stdin.take().expect("could not capture stdin");
        let stdout = child.stdout.take().expect("could not capture stdout");
        let stderr = child.stderr.take().expect("could not capture stderr");

        let stdout_stream = stream::StringStream::spawn(stdout);
        let stderr_stream = stream::StringStream::spawn(stderr);

        return Shell {
            stdin: stdin,
            stdout: stdout_stream,
            stderr: stderr_stream,
        };
    }

    pub fn execute(&mut self, command: &str) -> io::Result<()> {
        return self.stdin.write_all(command.as_ref());
    }

    pub fn poll_output(&self) -> io::Result<Option<String>> {
        match self.stderr.poll_output() {
            Ok(Some(string)) => {
                println!("stderr: \"{}\"", string);
            }
            Ok(None) => {
                // no output received
            }
            Err(e) => {
                println!("error:\n{}", e);
            }
        }

        return self.stdout.poll_output();
    }

    pub fn exit(mut self) -> io::Result<()> {
        return Err(io::Error::new(io::ErrorKind::Other, "not implemented"));
    }
}
