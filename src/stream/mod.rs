use std::sync::mpsc::{Receiver, Sender};

struct BufferedStream<I, O> {
    interruptor: Receiver<I>,
    output: Sender<O>,
}

impl<I, O> BufferedStream<I, O> {
    fn new(interruptor: Receiver<I>, output_channel: Sender<O>) -> Stream<I, O> {
        return Stream {
            interruptor: interruptor,
            output: output_channel,
        };
    }
}
