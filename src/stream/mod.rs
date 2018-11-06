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
use mio::unix::EventedFd;
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

            loop {
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
                    if event.token() == event_token && event.readiness() == Ready::readable() {
                        match reader.read(&mut result_buffer) {
                            Ok(0) => {
                                // nothing read
                            }
                            Ok(bytes_read) => {
                                // send buffer
                                if let Err(err) =
                                    stream_tx.send(Vec::from(&result_buffer[..bytes_read]))
                                {
                                    return Err(io::Error::new(
                                        io::ErrorKind::ConnectionAborted,
                                        format!("unable to send stream\n{}", err),
                                    ));
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        unreachable!()
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

    pub fn stop(&self) -> io::Result<()> {
        match self.signal_sender.send(Signal::Stop) {
            Ok(()) => {}
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "signal channel is closed",
                ));
            }
        }

        return Ok(());
    }
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

pub struct LockByteStream {
    thread_handle: JoinHandle<io::Result<()>>,
    receiver: Receiver<Vec<u8>>,
    lock: Arc<AtomicBool>,
}

impl LockByteStream {
    pub fn spawn<R: AsRawFd + Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();
        // let (lock_tx, lock_rx) = channel();

        let lock_test = Arc::new(AtomicBool::new(false));

        let child_lock = lock_test.clone();
        let thread_handle = spawn(move || {
            let mut byte_stream = ByteStream::spawn(readable);
            let mut buffer: Vec<u8> = Vec::new();
            let local_lock = child_lock;

            loop {
                match byte_stream.poll() {
                    Ok(Async::Ready(Some(mut bytes))) => {
                        buffer.append(&mut bytes);
                    }
                    Ok(Async::NotReady) => {}
                    Ok(Async::Ready(None)) => {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionRefused,
                            format!("unable to reach byte stream"),
                        ));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }

                if local_lock.load(Ordering::SeqCst) == true && !buffer.is_empty() {
                    let result = buffer.clone();
                    match stream_tx.send(result) {
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::ConnectionAborted,
                                format!("unable to send stream\n{}", err),
                            ))
                        }
                        Ok(()) => buffer.clear(),
                    }
                    local_lock.store(false, Ordering::SeqCst);
                }
            }
        });

        return LockByteStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
            lock: lock_test,
        };
    }

    pub fn send_buffer(&self) {
        self.lock.store(true, Ordering::SeqCst);
    }

    pub fn close(self) -> ::std::thread::Result<io::Result<()>> {
        return self.thread_handle.join();
    }
}

impl Stream for LockByteStream {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.lock.store(true, Ordering::SeqCst);
        match self.receiver.try_recv() {
            Ok(vec) => return Ok(Async::Ready(Some(vec))),
            Err(TryRecvError::Empty) => {
                return Ok(Async::NotReady);
            }
            Err(TryRecvError::Disconnected) => {
                return Ok(Async::Ready(None));
            }
        }
    }
}

fn debug_error<T>(msg: &'static str) -> io::Result<T> {
    return Err(io::Error::new(io::ErrorKind::Other, msg));
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{Async, Future};

    use std::io;
    use std::io::Read;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    const test_readable: &[u8] = "hello world".as_bytes();
    const SHORT_TIMEOUT_SECS: u64 = 1;
    const TIMEOUT_SECS: u64 = 3;

    #[test]
    fn test_create_byte_stream() {
        let mut stream = ByteStream::spawn(test_readable);
        let mut result: Vec<u8> = Vec::new();
        let expected = test_readable;

        let start_instant = Instant::now();

        sleep(Duration::from_secs(SHORT_TIMEOUT_SECS));
        stream.stop().expect("unable to send signal to stream");

        let result: Vec<u8> = stream
            .take(expected.len() as u64)
            .collect()
            .wait()
            .expect("could not collect output from stream");

        assert_eq!(&result, &expected);
    }

    #[test]
    fn test_create_stream() {
        let mut stream = LockByteStream::spawn(test_readable);

        let mut result: Vec<u8> = Vec::new();
        let expected = test_readable;

        sleep(Duration::from_secs(SHORT_TIMEOUT_SECS));
        stream.send_buffer();
        sleep(Duration::from_secs(SHORT_TIMEOUT_SECS));

        match stream.poll() {
            Ok(Async::Ready(Some(result))) => {
                assert_eq!(result.as_slice(), expected);
            }
            Ok(Async::NotReady) => {
                assert!(false, "stream has no output");
            }
            Ok(Async::Ready(None)) => {
                assert!(false, "error in stream thread");
            }
            Err(_) => {
                assert!(false, "undefined stream error");
            }
        };
    }
}
