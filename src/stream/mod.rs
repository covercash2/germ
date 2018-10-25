use std::io;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};
use std::thread::{spawn, JoinHandle};

use futures::{Async, Poll, Stream};

pub struct ByteStream {
    thread_handle: JoinHandle<io::Result<()>>,
    receiver: Receiver<Vec<u8>>,
    lock: Sender<()>,
}

impl ByteStream {
    pub fn spawn<R: Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();
        let (lock_tx, lock_rx) = channel();

        let thread_handle = spawn(move || {
            eprintln!("thread started");

            let mut reader = BufReader::new(readable);
            let mut buffer: Vec<u8> = Vec::new();
            let mut bytes = reader.bytes();

            loop {
                if let Some(Ok(byte)) = bytes.next() {
                    buffer.push(byte);
                };

                match lock_rx.try_recv() {
                    Ok(()) => {
                        eprintln!("lock opened");
                        let mut new_vec = Vec::new();
                        new_vec.append(&mut buffer);
                        match stream_tx.send(new_vec) {
                            Err(err) => {
                                return Err(io::Error::new(
                                    io::ErrorKind::ConnectionAborted,
                                    format!("unable to send stream\n{}", err),
                                ))
                            }
                            Ok(()) => {}
                        }
                    }
                    Err(TryRecvError::Disconnected) => {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionAborted,
                            format!("lock channel closed"),
                        ))
                    }
                    Err(TryRecvError::Empty) => {}
                }
            }

            eprintln!("thread finished");

            return Ok(());
        });

        return ByteStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
            lock: lock_tx,
        };
    }

    pub fn send_buffer(&self) {
        self.lock.send(());
    }

    pub fn close(self) -> ::std::thread::Result<io::Result<()>> {
        return self.thread_handle.join();
    }
}

impl Stream for ByteStream {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        eprintln!("polled");
        self.lock.send(());
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

    use futures::Async;

    use std::io;
    use std::io::Read;
    use std::thread::sleep;
    use std::time::Duration;

    const test_readable: &[u8] = "hello world".as_bytes();
    const timeout_secs: u64 = 3;

    #[test]
    fn test_create_stream() {
        let mut stream = ByteStream::spawn(test_readable);

        let mut result: Vec<u8> = Vec::new();
        let expected = test_readable;

        sleep(Duration::from_secs(timeout_secs / 2));
        stream.send_buffer();
        sleep(Duration::from_secs(timeout_secs / 2));

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
