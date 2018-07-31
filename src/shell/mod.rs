use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::spawn;
use std::thread::JoinHandle;

pub trait AsyncShell {
    type Buffer;
    type Error;
    fn send(&mut self, command: &str) -> Result<(), Self::Error>;
    fn receive(&self) -> Result<Self::Buffer, Self::Error>;
}

pub struct Shell {
    child: JoinHandle<()>,
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

        // TODO join properly
        let thread_handle = spawn(move || {
            let stdout = BufReader::new(stdout);
            for line in stdout.lines() {
                match line {
                    Ok(line) => output_tx.send(line).expect("could not send output"),
                    Err(ioerr) => eprintln!("warning: could not read line: {}", ioerr),
                }
            }
        });

        return Shell {
            child: thread_handle,
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

    pub fn exit(mut self) -> io::Result<()> {
        return self.child.join().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("unable to join child:\n{:?}", e),
            )
        });
    }
}
