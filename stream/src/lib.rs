#![feature(const_str_as_bytes)]
extern crate futures;
extern crate mio;

pub mod byte_stream;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
