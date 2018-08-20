use std::io;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::{spawn, JoinHandle};

pub struct AsyncStream {
    thread_handle: JoinHandle<Result<(), io::Error>>,
    receiver: Receiver<String>,
    lock: Sender<()>,
}

pub fn spawn_stream<R: Read + Send + 'static>(readable: R) -> AsyncStream {
    let mut reader = BufReader::new(readable);

    let (stream_tx, stream_rx) = channel();
    let (lock_tx, lock_rx) = channel();

    let mut buffer = String::new();

    let thread_handle = spawn(move || {
        loop {
            match reader.read_line(&mut buffer) {
                Ok(bytes_read) => {
                    match lock_rx.try_recv() {
                        Ok(()) => {
                            stream_tx.send(buffer.clone());
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
                                        "interruptor channel closed",
                                    ))
                                }
                            }
                        }
                    }
                }
                Err(ioerr) => return Err(ioerr),
            }
        }
    });

    return AsyncStream {
        thread_handle: thread_handle,
        receiver: stream_rx,
        lock: lock_tx,
    };
}
