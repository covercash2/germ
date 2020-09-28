use std::{io::Error as IoError, process::Command};

#[derive(Debug)]
enum ShellError {
    Io(IoError),
}

impl From<IoError> for ShellError {
    fn from(e: IoError) -> Self {
        ShellError::Io(e)
    }
}

trait Shell {
    fn run_command(&mut self, command: &str) -> Result<String, ShellError>;
}

struct DefaultShell;

impl Shell for DefaultShell {
    fn run_command(&mut self, command: &str) -> Result<String, ShellError> {
        let mut cmd_args = command.split_whitespace();

        let mut command = Command::new(cmd_args.next().unwrap_or_default());
        for arg in cmd_args {
            command.arg(arg);
        }
        let output = command.output()?;
        let stdout = String::from_utf8(output.stdout).expect("unable to create string from stdout");
        let stderr = String::from_utf8(output.stderr).expect("unable to create string from stderr");

	return Ok(stdout + &stderr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shell() {
	let mut shell = DefaultShell;
	let expected = "hello\n";
	let test = shell.run_command("echo hello").expect("unable to run shell command");
	println!("test: {:?}", test);
	assert!(test == expected);
    }
}
