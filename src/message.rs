use crate::{Header, MessageBody, QuestionsSection};
use core::mem::size_of;

pub trait Buffer {
    fn len(&self) -> usize;
    fn bytes(&self) -> &[u8];
    fn bytes_at(&self, offset: usize, size: usize) -> Result<&[u8], ()>;
}

pub trait MutBuffer: Buffer {
    fn bytes_mut(&mut self) -> &mut [u8];
}

pub trait ExtendableBuffer: MutBuffer {
    fn bytes_mut_at(&mut self, offset: usize, size: usize) -> Result<&mut [u8], ()>;
    fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), ()>;
    fn truncate(&mut self, len: usize);
}

impl<const CAP: usize> Buffer for arrayvec::ArrayVec<u8, CAP> {
    fn len(&self) -> usize {
        self.len()
    }

    fn bytes(&self) -> &[u8] {
        self.as_slice()
    }

    fn bytes_at(&self, offset: usize, size: usize) -> Result<&[u8], ()> {
        if offset + size > self.len() {
            return Err(());
        }

        Ok(&self.as_slice()[offset..offset + size])
    }
}

impl<const CAP: usize> MutBuffer for arrayvec::ArrayVec<u8, CAP> {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<const CAP: usize> ExtendableBuffer for arrayvec::ArrayVec<u8, CAP> {
    fn bytes_mut_at(&mut self, offset: usize, size: usize) -> Result<&mut [u8], ()> {
        if offset + size > self.len() {
            if offset + size > CAP {
                return Err(());
            }
            unsafe { self.set_len(offset + size) };
        }

        Ok(&mut self.as_mut_slice()[offset..offset + size])
    }

    fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), ()> {
        let pos = self.len();
        if pos + slice.len() > CAP {
            return Err(());
        }
        unsafe { self.set_len(pos + slice.len()) };
        self.as_mut_slice()[pos..].copy_from_slice(slice);
        Ok(())
    }

    fn truncate(&mut self, len: usize) {
        if len > self.len() {
            return;
        }
        unsafe { self.set_len(len) };
    }
}

impl Buffer for [u8] {
    fn len(&self) -> usize {
        self.len()
    }

    fn bytes(&self) -> &[u8] {
        self
    }

    fn bytes_at(&self, offset: usize, size: usize) -> Result<&[u8], ()> {
        if offset + size > self.len() {
            return Err(());
        }

        Ok(&self[offset..offset + size])
    }
}

/// A container for a mDNS message.
pub struct Message<'a, B: Buffer + ?Sized, const WRITE: bool> {
    _marker: core::marker::PhantomData<&'a B>,
    buffer: *const B,
}

impl<'a, B: Buffer + ?Sized> Message<'a, B, true> {
    /// Creates a new message.
    pub fn new_mut(buffer: &'a mut B) -> Result<Self, ()> {
        if buffer.len() > 0 {
            // Buffer must be empty.
            return Err(());
        }

        Ok(Self {
            _marker: core::marker::PhantomData,
            buffer: buffer as *const B,
        })
    }
}

impl<'a, B: Buffer + ?Sized> Message<'a, B, false> {
    /// Creates a new message.
    pub fn new(buffer: &'a B) -> Result<Self, ()> {
        if buffer.len() < size_of::<Header>() {
            // Buffer must be at least the size of a header.
            return Err(());
        }

        Ok(Self {
            _marker: core::marker::PhantomData,
            buffer: buffer as *const B,
        })
    }
}

impl<'a, B: Buffer + ?Sized, const WRITE: bool> Message<'a, B, WRITE> {
    /// Returns the message header.
    pub fn header(&self) -> Result<&Header, ()> {
        let bytes = unsafe { &*self.buffer }.bytes_at(0, size_of::<Header>())?;
        Ok(unsafe { &*(bytes.as_ptr() as *const Header) })
    }

    /// Returns the message body.
    pub fn body(&self) -> Result<MessageBody<'a, B, QuestionsSection, false>, ()> {
        let header = self.header()?;

        Ok(unsafe { MessageBody::new(header, self.buffer) })
    }
}

impl<'a, B: ExtendableBuffer + ?Sized> Message<'a, B, true> {
    /// Returns the message header.
    pub fn header_mut(&mut self) -> Result<&mut Header, ()> {
        let bytes =
            unsafe { &mut *(self.buffer as *mut B) }.bytes_mut_at(0, size_of::<Header>())?;
        Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut Header) })
    }

    /// Returns the message body.
    pub fn body_mut(&mut self) -> Result<MessageBody<'a, B, QuestionsSection, true>, ()> {
        let header = self.header_mut()?;

        Ok(unsafe { MessageBody::new(header, self.buffer) })
    }
}
