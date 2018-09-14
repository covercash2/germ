use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::iter::Take;
use std::path::PathBuf;
use std::process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender, TryIter, TryRecvError};
use std::thread::spawn;
use std::thread::JoinHandle;

pub mod stream;

pub struct Shell {
    stdin: ChildStdin,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl Write for Shell {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.stdin.write(buf);
    }
    fn flush(&mut self) -> io::Result<()> {
        return self.stdin.flush();
    }
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
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        return Shell {
            stdin: stdin,
            stdout: stdout,
            stderr: stderr,
        };
    }

    pub fn stdout(&mut self) -> Result<ChildStdout, &'static str> {
        return self.stdout.take().ok_or("stdout already taken from shell");
    }

    pub fn stderr(&mut self) -> Result<ChildStderr, &'static str> {
        return self.stderr.take().ok_or("stderr already taken from shell");
    }

    pub fn exit(mut self) -> io::Result<()> {
        return Err(io::Error::new(io::ErrorKind::Other, "not implemented"));
    }
}
