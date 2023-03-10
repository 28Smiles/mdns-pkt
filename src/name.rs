use core::fmt::{Display, Error, Formatter, Write};
use crate::ExtendableBuffer;

/// A DNS name.
#[derive(Debug, Clone, PartialEq)]
pub struct Name<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Name<'a> {
    pub fn parse(bytes: &'a [u8], i: &mut usize) -> Result<Self, ()> {
        const MAX_LENGTH: usize = 255;
        let mut j = *i;

        loop {
            if j - *i >= MAX_LENGTH {
                return Err(());
            }

            match LabelType::from_bytes(bytes, &mut j)? {
                LabelType::Pointer(_) => {
                    break;
                }
                LabelType::Part(len) => {
                    j += len as usize;

                    if len == 0 {
                        break;
                    }
                }
            }
        }

        let offset = *i;
        *i = j;

        Ok(Self { bytes, offset })
    }
}

impl PartialEq<[u8]> for Name<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        let mut i = self.offset;
        let mut depth = 0;
        let mut j = 0;
        loop {
            if depth > 255 {
                return false;
            }
            match LabelType::from_bytes(self.bytes, &mut i).unwrap() {
                LabelType::Pointer(ptr) => {
                    i = ptr as usize;
                }
                LabelType::Part(len) => {
                    if len == 0 {
                        return other.len() == j;
                    }

                    let part = &self.bytes[i..i + len as usize];
                    if depth > 0 {
                        if other[j] == b'.' {
                            j += 1;
                        } else {
                            return false;
                        }
                    }
                    let other = &other[j..j + len as usize];
                    if part != other {
                        return false;
                    }
                    i += len as usize;
                    j += len as usize;
                }
            }

            depth += 1;
        }
    }
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut i = self.offset;
        let mut depth = 0;
        loop {
            if depth > 255 {
                return Err(Error::default());
            }
            match LabelType::from_bytes(self.bytes, &mut i).unwrap() {
                LabelType::Pointer(ptr) => {
                    i = ptr as usize;
                }
                LabelType::Part(len) => {
                    if len == 0 {
                        return Ok(());
                    }

                    let part = &self.bytes[i..i + len as usize];
                    if depth > 0 {
                        f.write_char('.')?;
                    }
                    f.write_str(core::str::from_utf8(part).unwrap())?;
                    i += len as usize;
                }
            }

            depth += 1;
        }
    }
}

enum LabelType {
    Pointer(u16),
    Part(u8),
}

impl LabelType {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self, ()> {
        const PTR_MASK: u8 = 0b11000000;
        const LEN_MASK: u8 = !PTR_MASK;

        let c = bytes[*i];

        if c & PTR_MASK == PTR_MASK {
            let pointer = u16::from_be_bytes([c, bytes[1]]);
            if pointer > *i as u16 {
                // Cannot point to the future.
                return Err(());
            }

            *i += 2;
            Ok(Self::Pointer(pointer))
        } else {
            let len = c & LEN_MASK;
            *i += 1;

            Ok(Self::Part(len))
        }
    }
}

pub trait NamePart {
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()>;
}

pub struct NamePtr {
    offset: usize,
}

impl NamePart for NamePtr {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        (&self).to_bytes(buf)
    }
}

impl NamePart for &NamePtr {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        let offset = self.offset;
        let offset = offset as u16;
        let mut offset = offset.to_be_bytes();
        offset[0] |= 0b11000000; // Set the pointer bits.
        buf.extend_from_slice(&offset)
    }
}

impl NamePart for &[u8] {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        if self.len() > 63 {
            return Err(());
        }

        buf.extend_from_slice(&[self.len() as u8])?;
        buf.extend_from_slice(self)
    }
}

impl<const LEN: usize> NamePart for [u8; LEN] {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        (&self).to_bytes(buf)
    }
}

impl<const LEN: usize> NamePart for &[u8; LEN] {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        if LEN > 63 {
            return Err(());
        }

        buf.extend_from_slice(&[LEN as u8])?;
        buf.extend_from_slice(self)
    }
}

impl<'a> NamePart for Name<'a> {
    #[inline(always)]
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        (&self).to_bytes(buf)
    }
}

impl<'a> NamePart for &Name<'a> {
    fn to_bytes<B: ExtendableBuffer + ?Sized>(self, buf: &mut B) -> Result<(), ()> {
        let mut i = self.offset;
        let mut depth = 0;
        loop {
            if depth > 255 {
                return Err(());
            }
            match LabelType::from_bytes(self.bytes, &mut i).unwrap() {
                LabelType::Pointer(ptr) => {
                    i = ptr as usize;
                }
                LabelType::Part(len) => {
                    if len == 0 {
                        return Ok(());
                    }

                    let part = &self.bytes[i..i + len as usize];
                    buf.extend_from_slice(&[len])?;
                    buf.extend_from_slice(part)?;
                    i += len as usize;
                }
            }

            depth += 1;
        }
    }
}

pub struct NameBuilder<'a, B: ExtendableBuffer + ?Sized, P, O, F: Fn(P) -> O> {
    parent: P,
    finalizer: F,
    buffer: &'a mut B,
    last_offset: usize,
}

impl<'a, B: ExtendableBuffer + ?Sized, P, O, F: Fn(P) -> O> NameBuilder<'a, B, P, O, F> {
    pub(crate) fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> Self {
        let offset = buffer.len();
        Self {
            parent,
            finalizer,
            buffer,
            last_offset: offset,
        }
    }

    pub fn label<I: NamePart>(mut self, part: I) -> Result<Self, ()> {
        part.to_bytes(self.buffer)?;
        self.last_offset = self.buffer.len();

        Ok(self)
    }

    pub fn ptr(&self) -> NamePtr {
        NamePtr {
            offset: self.last_offset,
        }
    }

    pub fn finish(self) -> Result<O, ()> {
        // If the last label is not a pointer, add a null label.
        let end_of_name = self.buffer.len();
        let last_bytes = &self.buffer.bytes()[end_of_name - 2..end_of_name];
        if last_bytes[0] & 0b11000000 != 0b11000000 {
            self.buffer.extend_from_slice(&[0])?;
        }

        Ok((self.finalizer)(self.parent))
    }
}
