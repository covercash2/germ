use std::io;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

use futures::{Async, Poll, Stream};

struct ByteStream {
    thread_handle: JoinHandle<io::Result<()>>,
    receiver: Receiver<u8>,
}

impl ByteStream {
    pub fn spawn<R: Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();

        let thread_handle = spawn(move || {
            let reader = BufReader::new(readable);
            let mut bytes = reader.bytes();

            loop {
                if let Some(Ok(byte)) = bytes.next() {
                    match stream_tx.send(byte) {
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::ConnectionAborted,
                                format!("unable to send stream\n{}", err),
                            ))
                        }
                        Ok(()) => {}
                    }
                }
            }
        });

        return ByteStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
        };
    }
}

impl Stream for ByteStream {
    type Item = u8;
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
    pub fn spawn<R: Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();
        // let (lock_tx, lock_rx) = channel();

        let lock_test = Arc::new(AtomicBool::new(false));

        let child_lock = lock_test.clone();
        let thread_handle = spawn(move || {
            let reader = BufReader::new(readable);

            //let byte_stream = ByteStream::spawn(readeable);

            let mut buffer: Vec<u8> = Vec::new();
            let mut bytes = reader.bytes();
            let local_lock = child_lock;

            loop {
                if let Some(Ok(byte)) = bytes.next() {
                    buffer.push(byte);
                };

                if local_lock.load(Ordering::SeqCst) == true {
                    let result = buffer.clone();

                    eprintln!(
                        "sending buffer: {}",
                        String::from_utf8(result.clone()).unwrap()
                    );

                    match stream_tx.send(result) {
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::ConnectionAborted,
                                format!("unable to send stream\n{}", err),
                            ))
                        }
                        Ok(()) => {
                            // local_lock.store(false, Ordering::SeqCst);
                            buffer.clear()
                        }
                    }
                } else {
                    eprintln!("not polled");
                }
            }
        });

        return LockByteStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
            lock: lock_test,
        };
    }

    // pub fn send_buffer(&self) {
    //     self.lock.send(());
    // }

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

    use futures::Async;

    use std::io;
    use std::io::Read;
    use std::thread::sleep;
    use std::time::Duration;

    const test_readable: &[u8] = "hello world".as_bytes();
    const timeout_secs: u64 = 3;

    #[test]
    fn test_create_stream() {
        let mut stream = LockByteStream::spawn(test_readable);

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
