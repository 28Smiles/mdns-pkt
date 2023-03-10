use crate::builder::answer::type_a::AnswerTypeABuilder;
use crate::builder::answer::type_ptr::AnswerTypePtrBuilder;
use crate::{AnswerTypeTxtBuilder, ExtendableBuffer};

pub struct AnswerTypeBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> {
    parent: P,
    finalizer: F,
    buffer: &'a mut B,
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerTypeBuilder<'a, B, P, O, F> {
    #[inline(always)]
    pub(crate) fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerTypeBuilder<'a, B, P, O, F> {
        AnswerTypeBuilder { parent, finalizer, buffer }
    }

    #[inline(always)]
    pub fn a(self) -> AnswerTypeABuilder<'a, B, P, O, F, false, > {
        let buffer_pos = self.buffer.len();
        self.buffer.bytes_mut_at(buffer_pos - 9, 1).unwrap()
            .copy_from_slice(&[1]);
        AnswerTypeABuilder::new(self.buffer, self.parent, self.finalizer)
    }

    #[inline(always)]
    pub fn ptr(self) -> AnswerTypePtrBuilder<'a, B, P, O, F, false, > {
        let buffer_pos = self.buffer.len();
        self.buffer.bytes_mut_at(buffer_pos - 9, 1).unwrap()
            .copy_from_slice(&[12]);
        AnswerTypePtrBuilder::new(self.buffer, self.parent, self.finalizer)
    }

    #[inline(always)]
    pub fn txt(self) -> AnswerTypeTxtBuilder<'a, B, P, O, F, false, > {
        let buffer_pos = self.buffer.len();
        self.buffer.bytes_mut_at(buffer_pos - 9, 1).unwrap()
            .copy_from_slice(&[16]);
        AnswerTypeTxtBuilder::new(self.buffer, self.parent, self.finalizer)
    }
}
