use std::mem::size_of;

use byteorder::{ByteOrder, LittleEndian};

pub use memdump_derive::{Dump, FromDump};

pub trait Dump: Sized {
    fn dump(&self, buf: &mut [u8]) -> usize;
}

pub trait FromDump: Sized {
    fn from_dump(buf: &[u8]) -> (Self, usize);
}

impl Dump for i8 {
    fn dump(&self, buf: &mut [u8]) -> usize {
        buf[0] = *self as u8;
        size_of::<Self>()
    }
}

impl FromDump for i8 {
    fn from_dump(buf: &[u8]) -> (Self, usize) {
        (buf[0] as i8, size_of::<Self>())
    }
}

impl Dump for u32 {
    fn dump(&self, buf: &mut [u8]) -> usize {
        LittleEndian::write_u32(buf, *self);
        size_of::<Self>()
    }
}

impl FromDump for u32 {
    fn from_dump(buf: &[u8]) -> (Self, usize) {
        (LittleEndian::read_u32(buf), size_of::<Self>())
    }
}
