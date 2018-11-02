use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};

use futures::{Async, Stream};

use stream;

type ShellError = ::std::option::NoneError;

pub struct Shell {
    child: Child,
    stdin: ChildStdin,
    stdout: stream::LockByteStream,
    stderr: stream::LockByteStream,
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

        let stdout_stream = stream::LockByteStream::spawn(stdout);
        let stderr_stream = stream::LockByteStream::spawn(stderr);

        return Ok(Shell {
            child: child,
            stdin: stdin,
            stdout: stdout_stream,
            stderr: stderr_stream,
        });
    }

    pub fn execute(&mut self, command: &str) -> io::Result<()> {
        return self.stdin.write_all(command.as_ref());
    }

    pub fn poll_stdout(&mut self) -> io::Result<Option<Vec<u8>>> {
        match self.stdout.poll() {
            Ok(Async::Ready(Some(result))) => {
                return Ok(Some(result));
            }
            Ok(Async::NotReady) => {
                return Ok(None);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "could not access stream",
                ));
            }
        }
    }

    pub fn exit(&mut self) -> io::Result<()> {
        return self.child.kill();
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
        let max_iters = 1000000;
        let mut iters = 0;
        while (!buffer.eq(expected_output)) {
            iters += 1;
            if (iters >= max_iters) {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("too many iterations\nbuffer: {}", buffer),
                ));
            }
            match shell.poll_stdout() {
                Ok(output) => output.map(|s| {
                    let s = String::from_utf8(s).expect("could not parse shell stdout");
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
