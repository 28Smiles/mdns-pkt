mod builder;
mod type_a;
mod type_ptr;
mod type_txt;

pub use builder::*;
pub use type_a::*;
pub use type_ptr::*;
pub use type_txt::*;

use crate::{AClass, ExtendableBuffer, NameBuilder};
use crate::builder::answer::builder::AnswerTypeBuilder;

pub struct AnswerBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    // THE STATE
    const NAME: bool,
    const CACHE_FLUSH: bool,
    const TYPE: bool,
    const CLASS: bool,
    const TTL: bool,
> {
    parent: P,
    finalizer: F,
    name_end: usize,
    buffer: &'a mut B,
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerBuilder<'a, B, P, O, F, false, false, false, false, false> {
    #[inline(always)]
    pub fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerBuilder<
        'a, B, P, O, F,
        false,
        false,
        false,
        false,
        false,
    > {
        AnswerBuilder { parent, finalizer, name_end: 0, buffer }
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerBuilder<'a, B, P, O, F, false, false, false, false, false> {
    #[inline(always)]
    pub fn name(self) -> NameBuilder<
        'a,
        B,
        Self,
        Result<AnswerBuilder<'a, B, P, O, F, true, false, false, false, false>, ()>,
        fn(AnswerBuilder<'a, B, P, O, F, false, false, false, false, false>) -> Result<AnswerBuilder<'a, B, P, O, F, true, false, false, false, false>, ()>,
    >
    {
        let buffer_ptr = unsafe { &mut *(self.buffer as *mut B) };
        NameBuilder::new(buffer_ptr, self, |parent| {
            let name_end = parent.buffer.len();
            // Insert empty bytes for class, type, ttl, and length
            parent.buffer.extend_from_slice(&[0; 10])
                .map(|_| AnswerBuilder {
                    parent: parent.parent,
                    finalizer: parent.finalizer,
                    name_end,
                    buffer: parent.buffer,
                })
        })
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const CACHE_FLUSH: bool,
    const TYPE: bool,
    const CLASS: bool,
    const TTL: bool,
> AnswerBuilder<'a, B, P, O, F, true, CACHE_FLUSH, TYPE, CLASS, TTL> {
    #[inline(always)]
    pub fn cache_flush(self, value: bool) -> Result<AnswerBuilder<
        'a, B, P, O, F,
        true,
        true,
        TYPE,
        CLASS,
        TTL,
    >, ()> {
        // Set the cache flush bit if true and clear it if false.
        let byte = self.buffer.bytes_mut_at(self.name_end, 1)?;
        byte[0] = (byte[0] & 0b0111_1111) | ((value as u8) << 7);

        Ok(AnswerBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            name_end: self.name_end,
            buffer: self.buffer,
        })
    }

    #[inline(always)]
    pub fn ttl(self, value: u32) -> Result<AnswerBuilder<
        'a, B, P, O, F,
        true,
        CACHE_FLUSH,
        TYPE,
        CLASS,
        true,
    >, ()> {
        self.buffer.bytes_mut_at(self.name_end + 4, 4)?
            .copy_from_slice(&value.to_be_bytes());

        Ok(AnswerBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            name_end: self.name_end,
            buffer: self.buffer,
        })
    }

    #[inline(always)]
    pub fn aclass(self, value: AClass) -> Result<AnswerBuilder<
        'a, B, P, O, F,
        true,
        CACHE_FLUSH,
        TYPE,
        true,
        TTL,
    >, ()> {
        let value: u16 = value.into();
        let mut value = value.to_be_bytes();
        value[0] &= 0b0111_1111; // Clear the cache flush bit.
        value[0] |= self.buffer.bytes()[self.name_end] & 0b1000_0000; // Copy the cache flush bit.
        self.buffer.bytes_mut_at(self.name_end + 2, 2)?
            .copy_from_slice(&value);

        Ok(AnswerBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            name_end: self.name_end,
            buffer: self.buffer,
        })
    }

    #[inline(always)]
    pub fn atype(self) -> AnswerTypeBuilder<
        'a, B,
        AnswerBuilder<'a, B, P, O, F, true, CACHE_FLUSH, TYPE, CLASS, TTL>,
        Result<AnswerBuilder<'a, B, P, O, F, true, CACHE_FLUSH, true, CLASS, TTL>, ()>,
        fn(AnswerBuilder<'a, B, P, O, F, true, CACHE_FLUSH, TYPE, CLASS, TTL>) -> Result<AnswerBuilder<'a, B, P, O, F, true, CACHE_FLUSH, true, CLASS, TTL>, ()>,
    > {
        let buffer_ptr = unsafe { &mut *(self.buffer as *mut B) };
        AnswerTypeBuilder::new(buffer_ptr, self, |parent| {
            // Write the length of the answer data.
            let bytes_written = parent.buffer.len() - parent.name_end - 10;
            let bytes_written = bytes_written as u16;
            parent.buffer.bytes_mut_at(parent.name_end + 8, 2)
                .map(|bytes| bytes.copy_from_slice(&bytes_written.to_be_bytes()))
                .map(|_| AnswerBuilder {
                    parent: parent.parent,
                    finalizer: parent.finalizer,
                    name_end: parent.name_end,
                    buffer: parent.buffer,
                })
        })
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerBuilder<'a, B, P, O, F, true, true, true, true, true> {
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}