use crate::{ExtendableBuffer, NameBuilder};

pub struct AnswerTypeSrvBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const PRIORITY: bool,
    const WEIGHT: bool,
    const PORT: bool,
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
> AnswerTypeSrvBuilder<'a, B, P, O, F, false, false, false, false> {
    #[inline(always)]
    pub fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> AnswerTypeSrvBuilder<
        'a, B, P, O, F,
        false,
        false,
        false,
        false,
    > {
        let start = buffer.len();
        AnswerTypeSrvBuilder { parent, finalizer, buffer, start }
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const PRIORITY: bool,
    const WEIGHT: bool,
    const PORT: bool,
    const NAME: bool,
> AnswerTypeSrvBuilder<'a, B, P, O, F, PRIORITY, WEIGHT, PORT, NAME> {
    #[inline(always)]
    pub fn priority(self, priority: u16) -> Result<AnswerTypeSrvBuilder<
        'a, B, P, O, F,
        true,
        WEIGHT,
        PORT,
        NAME,
    >, ()> {
        self.buffer.bytes_mut_at(self.start, 2)?.copy_from_slice(&priority.to_be_bytes());

        Ok(AnswerTypeSrvBuilder { parent: self.parent, finalizer: self.finalizer, buffer: self.buffer, start: self.start })
    }

    #[inline(always)]
    pub fn weight(self, weight: u16) -> Result<AnswerTypeSrvBuilder<
        'a, B, P, O, F,
        PRIORITY,
        true,
        PORT,
        NAME,
    >, ()> {
        self.buffer.bytes_mut_at(self.start + 2, 2)?.copy_from_slice(&weight.to_be_bytes());

        Ok(AnswerTypeSrvBuilder { parent: self.parent, finalizer: self.finalizer, buffer: self.buffer, start: self.start })
    }

    #[inline(always)]
    pub fn port(self, port: u16) -> Result<AnswerTypeSrvBuilder<
        'a, B, P, O, F,
        PRIORITY,
        WEIGHT,
        true,
        NAME,
    >, ()> {
        self.buffer.bytes_mut_at(self.start + 4, 2)?.copy_from_slice(&port.to_be_bytes());

        Ok(AnswerTypeSrvBuilder { parent: self.parent, finalizer: self.finalizer, buffer: self.buffer, start: self.start })
    }

    #[inline(always)]
    pub fn name(self) -> NameBuilder<
        'a,
        B,
        Self,
        AnswerTypeSrvBuilder<'a, B, P, O, F, PRIORITY, WEIGHT, PORT, true>,
        fn(AnswerTypeSrvBuilder<'a, B, P, O, F, PRIORITY, WEIGHT, PORT, NAME>) -> AnswerTypeSrvBuilder<'a, B, P, O, F, PRIORITY, WEIGHT, PORT, true>,
    > {
        let buffer_ptr = unsafe { &mut *(self.buffer as *mut B) };
        buffer_ptr.truncate(self.start + 6); // In case we already have a name we need to truncate it

        NameBuilder::new(
            buffer_ptr,
            self,
            |builder| {
                AnswerTypeSrvBuilder {
                    parent: builder.parent,
                    finalizer: builder.finalizer,
                    buffer: builder.buffer,
                    start: builder.start
                }
            },
        )
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> AnswerTypeSrvBuilder<'a, B, P, O, F, true, true, true, true> {
    #[inline(always)]
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}