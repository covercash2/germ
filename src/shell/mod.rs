use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::iter::Take;
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender, TryIter, TryRecvError};
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
            .stderr(Stdio::piped())
            .spawn()
            .expect("should work dummy");

        let stdin = child.stdin.take().expect("could not capture stdin");
        let stdout = child.stdout.take().expect("could not capture stdout");
        let stderr = child.stderr.take().expect("could not capture stderr");

        let (stdout_tx, stdout_rx) = channel();

        // TODO join properly
        let thread_handle = spawn(move || {
            let stdout = BufReader::new(stdout);

            // TODO parsing lines is taking a while
            for line in stdout.lines() {
                match line {
                    Ok(line) => stdout_tx.send(line).expect("could not send output"),
                    Err(ioerr) => eprintln!("warning: could not read line: {}", ioerr),
                }
            }
        });

        // let stderr_handle = spawn(move || {
        //     let stderr = BufReader::new(stderr);
        //     return send_stream(stderr);
        // });

        return Shell {
            child: thread_handle,
            stdin: stdin,
            stdout: stdout_rx,
        };
    }

    pub fn execute(&mut self, command: &str) -> io::Result<()> {
        return self.stdin.write_all(command.as_ref());
    }

    pub fn poll_output(&self, lines: usize) -> Take<TryIter<String>> {
        return self.stdout.try_iter().take(lines);
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

/// continuously buffer output from a stream until
/// interrupted. then, send output along an output channel.
fn send_stream<R: Read>(
    stream: R,
    sender: Sender<String>,
    interruptor: Receiver<()>,
) -> io::Result<()> {
    let mut stream = BufReader::new(stream);

    let mut buffer = String::new();

    while let Ok(bytes_read) = stream.read_line(&mut buffer) {
        match interruptor.try_recv() {
            Ok(()) => {
                // interrupted
                // send output
                sender.send(buffer.clone());
                buffer.clear();
            }
            Err(TryRecvError::Empty) => {
                // not interrupted
                // continue to buffer output
            }
            Err(TryRecvError::Disconnected) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "interruptor channel closed",
                ));
            }
        }
    }

    return Ok(());
}
