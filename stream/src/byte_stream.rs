use std::io;
use std::io::{BufReader, Read};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

use futures::{Async, Poll, Stream};

use mio::event::Evented;
use mio::unix::{EventedFd, UnixReady};
use mio::Poll as EventPoll;
use mio::{Events, PollOpt, Ready, Token};

const DEFAULT_BUFFER_SIZE: usize = 64;

pub enum Signal {
    Stop,
}

struct ByteStream {
    thread_handle: JoinHandle<io::Result<()>>,
    receiver: Receiver<Vec<u8>>,
    signal_sender: Sender<Signal>,
}

impl ByteStream {
    pub fn spawn<R: AsRawFd + Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();
        let (signal_tx, signal_rx) = channel();

        let thread_handle = spawn(move || {
            let fd = readable.as_raw_fd();
            let mut reader = BufReader::new(readable);
            let mut result_buffer: [u8; DEFAULT_BUFFER_SIZE] = [0; DEFAULT_BUFFER_SIZE];

            let event_poll = EventPoll::new().expect("could not create event poll");
            let event_token_token = 0;
            let event_token = Token(event_token_token);

            event_poll.register(
                &EventedFd(&fd),
                event_token,
                Ready::readable(),
                PollOpt::level(),
            )?;

            const EVENTS_CAPACITY: usize = 8;

            let mut events = Events::with_capacity(EVENTS_CAPACITY);

            'thread: loop {
                // TODO magic numbers
                event_poll
                    .poll(&mut events, Some(Duration::from_millis(10)))
                    .expect("unable to poll for events");

                match signal_rx.try_recv() {
                    Ok(Signal::Stop) => {
                        return Ok(());
                    }
                    Err(TryRecvError::Disconnected) => {
                        return Err(io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "signal channel is disconnected",
                        ));
                    }
                    _ => {
                        // empty signal queue
                    }
                }

                for event in events.iter() {
                    debug!("event: {:?}", event);
                    if event.token() == event_token && event.readiness().is_readable() {
                        match read(&mut reader, &mut result_buffer) {
                            Ok(None) => {
                                debug!("nothing read");
                            }
                            Ok(Some(vec)) => {
                                debug!("output: {}", ::std::str::from_utf8(&vec).unwrap());
                                if let Err(err) = stream_tx.send(vec) {
                                    return Err(io::Error::new(
                                        io::ErrorKind::ConnectionAborted,
                                        format!("unable to send stream:\n\t{}", err),
                                    ));
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else if UnixReady::from(event.readiness()) == UnixReady::hup() {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionAborted,
                            "poll has hung up",
                        ));
                    }
                }
            }
        });

        return ByteStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
            signal_sender: signal_tx,
        };
    }

    pub fn stop(self) -> io::Result<()> {
        match self.signal_sender.send(Signal::Stop) {
            Ok(()) => {}
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "signal channel is closed",
                ));
            }
        }

        return match self.thread_handle.join() {
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("stream thread panicked:\n\t{:?}", e),
            )),
            Ok(output) => {
                return output;
            }
        };
    }
}

fn read<R: Read>(reader: &mut R, buffer: &mut [u8]) -> io::Result<Option<Vec<u8>>> {
    return match reader.read(buffer) {
        Ok(0) => Ok(None),
        Ok(bytes_read) => Ok(Some(Vec::from(&mut buffer[..bytes_read]))),
        Err(e) => Err(e),
    };
}

impl Stream for ByteStream {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.receiver.try_recv() {
            Ok(t) => return Ok(Async::Ready(Some(t))),
            Err(TryRecvError::Empty) => {
                return Ok(Async::NotReady);
            }
            Err(TryRecvError::Disconnected) => {
                return Ok(Async::Ready(None));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{Async, Future};

    use simple_logger;

    use std::io;
    use std::io::{Read, Write};
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::str::from_utf8;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    // setup logger
    use std::sync::{Once, ONCE_INIT};
    static LOG_INIT: Once = ONCE_INIT;
    fn setup() {
        LOG_INIT.call_once(|| {
            simple_logger::init().expect("unable to init logger");
        });
    }

    const TEST_COMMAND: &str = "echo";
    const TEST_STRING: &str = "hello, world";
    const TEST_OUTPUT: &str = "hello, world\n";
    const TEST_READABLE: &[u8] = TEST_STRING.as_bytes();
    const SHORT_TIMEOUT_SECS: u64 = 1;
    const TIMEOUT_SECS: u64 = 3;

    fn spawn_test_command() -> Child {
        return Command::new("sh")
            .arg("-c")
            .arg(&format!("{} {}", TEST_COMMAND, TEST_STRING))
            .stdout(Stdio::piped())
            .spawn()
            .expect("could not spawn test command");
    }

    fn spawn_test_child() -> Child {
        return Command::new("sh")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("could not spawn test child");
    }

    #[test]
    fn test_create_byte_stream() {
        setup();

        let mut child = spawn_test_child();
        let mut stdin = child.stdin.take().expect("could not take stdin from child");
        let _bytes_written = stdin
            .write(&format!("{} {}", TEST_COMMAND, TEST_STRING).as_bytes())
            .expect("could not write to stdin");
        let mut stream = ByteStream::spawn(
            child
                .stdout
                .take()
                .expect("could not get stdout from child"),
        );
        let expected = format!("{}\n", TEST_STRING);

        match stream.poll() {
            Ok(Async::Ready(Some(result))) => {
                assert_eq!(
                    &from_utf8(&result).expect("couldn't parse output"),
                    &expected
                );
            }
            Ok(Async::Ready(None)) => {
                panic!("stream has been closed");
            }
            _ => debug!("output not ready"),
        }

        stream.stop().expect("error stopping stream");
    }
}
