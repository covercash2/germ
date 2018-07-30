use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread::spawn;

type Error = String;

pub trait AsyncShell {
    type Buffer;
    type Error;
    fn send(&mut self, command: &str) -> Result<(), Self::Error>;
    fn receive(&self) -> Result<Self::Buffer, Self::Error>;
}

pub struct Shell {
    bin_path: PathBuf,
    process: Child,
    stdin: ChildStdin,
    stdout: Receiver<String>,
}

impl Shell {
    pub fn new(bin_path: PathBuf) -> Shell {
        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("should work dummy");

        let stdin = child.stdin.take().expect("could not capture stdin");
        let stdout = child.stdout.take().expect("could not capture stdout");

        let (output_tx, output_rx) = channel();

        let thread = spawn(move || {
            let stdout = BufReader::new(stdout);
            for line in stdout.lines() {
                match line {
                    Ok(line) => output_tx.send(line).expect("could not send output"),
                    Err(ioerr) => eprintln!("warning: could not read line: {}", ioerr),
                }
            }
        });

        return Shell {
            bin_path: bin_path,
            process: child,
            stdin: stdin,
            stdout: output_rx,
        };
    }

    pub fn execute(&mut self, command: &str) -> io::Result<()> {
        return self.stdin.write_all(command.as_ref());
    }

    pub fn poll_output(&self) -> Option<String> {
        match self.stdout.try_recv() {
            Ok(line) => return Some(line),
            Err(e) => match e {
                TryRecvError::Empty => return None,
                TryRecvError::Disconnected => {
                    eprintln!("warning: output thread is disconnected");
                    return None;
                }
            },
        }
    }

    // TODO safely exit
    // pub fn exit(mut self) -> io::Result<()> {
    // }
}

fn not_found_error(msg: &str) -> io::Error {
    return io::Error::new(io::ErrorKind::NotFound, msg);
}
