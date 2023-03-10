use crate::{ExtendableBuffer, NameBuilder, QClass, QType};

pub struct QuestionBuilder<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    // THE STATE
    const NAME: bool,
    const TYPE: bool,
    const CLASS: bool,
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
> QuestionBuilder<'a, B, P, O, F, false, false, false> {
    #[inline(always)]
    pub fn new(
        buffer: &'a mut B,
        parent: P,
        finalizer: F,
    ) -> QuestionBuilder<
        'a, B, P, O, F,
        false,
        false,
        false,
    > {
        QuestionBuilder { parent, finalizer, name_end: 0, buffer }
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> QuestionBuilder<'a, B, P, O, F, false, false, false> {
    #[inline(always)]
    pub fn name(self) -> NameBuilder<
        'a,
        B,
        Self,
        QuestionBuilder<'a, B, P, O, F, true, false, false>,
        fn(QuestionBuilder<'a, B, P, O, F, false, false, false>) -> QuestionBuilder<'a, B, P, O, F, true, false, false>,
    >
    {
        let buffer_ptr = unsafe { &mut *(self.buffer as *mut B) };
        NameBuilder::new(buffer_ptr, self, |parent|
            QuestionBuilder {
                parent: parent.parent,
                finalizer: parent.finalizer,
                name_end: parent.buffer.len(),
                buffer: parent.buffer,
            }
        )
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
    const TYPE: bool,
    const CLASS: bool,
> QuestionBuilder<'a, B, P, O, F, true, TYPE, CLASS> {
    #[inline(always)]
    pub fn qclass(self, value: QClass) -> Result<QuestionBuilder<
        'a, B, P, O, F,
        true,
        TYPE,
        true,
    >, ()> {
        let value: u16 = value.into();
        self.buffer.bytes_mut_at(self.name_end + 2, 2)?
            .copy_from_slice(&value.to_be_bytes());

        Ok(QuestionBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            name_end: self.name_end,
            buffer: self.buffer,
        })
    }

    #[inline(always)]
    pub fn qtype(self, value: QType) -> Result<QuestionBuilder<
        'a, B, P, O, F,
        true,
        true,
        CLASS,
    >, ()> {
        let value: u16 = value.into();
        self.buffer.bytes_mut_at(self.name_end, 2)?
            .copy_from_slice(&value.to_be_bytes());

        Ok(QuestionBuilder {
            parent: self.parent,
            finalizer: self.finalizer,
            name_end: self.name_end,
            buffer: self.buffer,
        })
    }
}

impl<
    'a,
    B: ExtendableBuffer + ?Sized,
    P, O, F: Fn(P) -> O,
> QuestionBuilder<'a, B, P, O, F, true, true, true> {
    pub fn finish(self) -> O {
        (self.finalizer)(self.parent)
    }
}