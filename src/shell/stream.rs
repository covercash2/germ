use std::io;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::{spawn, JoinHandle};

use std::time::Instant;

pub struct StringStream {
    thread_handle: JoinHandle<Result<(), io::Error>>,
    receiver: Receiver<String>,
    lock: Sender<()>,
}

impl StringStream {
    pub fn spawn<R: Read + Send + 'static>(readable: R) -> Self {
        let (stream_tx, stream_rx) = channel();
        let (lock_tx, lock_rx) = channel();

        let thread_handle = spawn(move || {
            let mut reader = BufReader::new(readable);
            let mut buffer = String::new();

            loop {
                send_output(&mut buffer, &mut reader, &lock_rx, &stream_tx);
            }
        });

        return StringStream {
            thread_handle: thread_handle,
            receiver: stream_rx,
            lock: lock_tx,
        };
    }

    pub fn poll_output(&self) -> Result<Option<String>, io::Error> {
        self.lock.send(());
        match self.receiver.try_recv() {
            Ok(output) => return Ok(Some(output)),
            Err(e) => {
                match e {
                    TryRecvError::Empty => {
                        // buffer is empty
                        return Ok(None);
                    }
                    TryRecvError::Disconnected => {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionAborted,
                            format!("stream has been aborted:\n{}", e),
                        ))
                    }
                }
            }
        }
    }
}

fn send_output(
    buffer: &mut String,
    ref mut reader: impl BufRead,
    lock: &Receiver<()>,
    sender: &Sender<String>,
) -> io::Result<()> {
    match reader.read_line(buffer) {
        Ok(bytes_read) => {
            match lock.try_recv() {
                Ok(()) => {
                    sender.send(buffer.clone());
                    buffer.clear();
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {
                            // not interrupted
                            // do nothing
                            // continue to buffer
                        }
                        TryRecvError::Disconnected => {
                            return Err(io::Error::new(
                                io::ErrorKind::ConnectionAborted,
                                format!("interruptor channel closed:\n{}", e),
                            ))
                        }
                    }
                }
            }
        }
        Err(ioerr) => return Err(ioerr),
    }

    return Ok(());
}
