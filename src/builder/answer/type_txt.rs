use crate::ExtendableBuffer;

pub struct AnswerTypeTxtBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const TXT: bool,
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
    const TXT: bool,
> AnswerTypeTxtBuilder<'a, B, P, O, F, TXT> {
    #[inline(always)]
    pub(crate) fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerTypeTxtBuilder<'a, B, P, O, F, TXT> {
        let start = buffer.len();
        AnswerTypeTxtBuilder { parent, finalizer, buffer, start }
    }

    #[inline(always)]
    pub fn txt(self, txt: &[u8]) -> Result<AnswerTypeTxtBuilder<'a, B, P, O, F, true>, ()> {
        self.buffer.bytes_mut_at(self.start, txt.len())?.copy_from_slice(txt);

        Ok(AnswerTypeTxtBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            buffer: self.buffer,
            start: self.start
        })
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerTypeTxtBuilder<'a, B, P, O, F, true> {
    #[inline(always)]
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}
