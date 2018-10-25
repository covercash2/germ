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

type ShellError = ::std::option::NoneError;

pub struct Shell {
    stdin: ChildStdin,
    stdout: stream::StringStream,
    stderr: stream::StringStream,
}

impl Shell {
    pub fn create(bin_path: PathBuf) -> Result<Shell, ShellError> {
        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("should work dummy");

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let stderr = child.stderr.take()?;

        let stdout_stream = stream::StringStream::spawn(stdout);
        let stderr_stream = stream::StringStream::spawn(stderr);

        return Ok(Shell {
            stdin: stdin,
            stdout: stdout_stream,
            stderr: stderr_stream,
        });
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::io;

    use test::Bencher;

    const BASH_SHELL_PATH: &str = "/bin/bash";

    fn test_shell() -> Shell {
        return Shell::create(BASH_SHELL_PATH.into()).expect("could not create test `bash` shell");
    }

    #[test]
    fn test_create() {
        test_shell();
    }

    fn test_command_with_known_output(
        shell: &mut Shell,
        command: &str,
        expected_output: &String,
    ) -> io::Result<()> {
        shell.execute(command);
        let mut buffer: String = String::new();
        let max_iters = 10000;
        let mut iters = 0;
        while (!buffer.eq(expected_output)) {
            iters += 1;
            if (iters >= max_iters) {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("too many iterations\nbuffer: {}", buffer),
                ));
            }
            match shell.poll_output() {
                Ok(output) => output.map(|s| {
                    buffer.push_str(&s);
                }),
                Err(ioerr) => return Err(ioerr),
            };
        }
        return Ok(());
    }

    fn hello_world(shell: &mut Shell) -> io::Result<()> {
        let expected_output = "hello world\n".to_string();
        let command = "echo hello world\n";

        return test_command_with_known_output(shell, command, &expected_output);
    }

    #[test]
    fn test_hello_world() {
        hello_world(&mut test_shell()).expect("could not run hello world");
    }

    #[test]
    fn test_usr_bin() {}

    #[bench]
    fn bench_send(bencher: &mut Bencher) {
        let mut shell: Shell = test_shell();

        bencher.iter(|| {
            hello_world(&mut shell);
        })
    }
}
