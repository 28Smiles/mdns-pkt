use crate::ExtendableBuffer;

pub struct AnswerTypeABuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const IP: bool,
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
    const IP: bool,
> AnswerTypeABuilder<'a, B, P, O, F, IP> {
    #[inline(always)]
    pub(crate) fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerTypeABuilder<'a, B, P, O, F, IP> {
        let start = buffer.len();
        AnswerTypeABuilder { parent, finalizer, buffer, start }
    }

    #[inline(always)]
    pub fn ip(self, ip: &[u8; 4]) -> Result<AnswerTypeABuilder<'a, B, P, O, F, true>, ()> {
        self.buffer.bytes_mut_at(self.start, 4)?.copy_from_slice(ip);
        Ok(AnswerTypeABuilder {
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
> AnswerTypeABuilder<'a, B, P, O, F, true> {
    #[inline(always)]
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}
