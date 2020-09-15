#![allow(dead_code)]
use std::io;

use bytes::{BufMut, BytesMut};

pub const SIZE: usize = 27;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: &'static str,
}

pub struct Writer<'a>(pub &'a mut BytesMut);

impl<'a> io::Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.put_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
