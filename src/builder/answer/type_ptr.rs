use crate::{ExtendableBuffer, NameBuilder};

pub struct AnswerTypePtrBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const NAME: bool,
> {
    parent: P,
    finalizer: F,
    buffer: &'a mut B,
    start: usize,
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerTypePtrBuilder<'a, B, P, O, F, false> {
    #[inline(always)]
    pub(crate) fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerTypePtrBuilder<'a, B, P, O, F, false> {
        let start = buffer.len();
        AnswerTypePtrBuilder { parent, finalizer, buffer, start }
    }

    #[inline(always)]
    pub fn name(self) -> NameBuilder<
        'a,
        B,
        Self,
        AnswerTypePtrBuilder<'a, B, P, O, F, true>,
        fn(AnswerTypePtrBuilder<'a, B, P, O, F, false>) -> AnswerTypePtrBuilder<'a, B, P, O, F, true>,
    >
    {
        let buffer_ptr = unsafe { &mut *(self.buffer as *mut B) };
        NameBuilder::new(buffer_ptr, self, |parent|
            AnswerTypePtrBuilder {
                parent: parent.parent,
                finalizer: parent.finalizer,
                buffer: parent.buffer,
                start: parent.start,
            }
        )
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerTypePtrBuilder<'a, B, P, O, F, true> {
    #[inline(always)]
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}