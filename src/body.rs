use crate::{
    AClass, AType, Answer, Buffer, ExtendableBuffer, Header, NameLabel, QClass, QType, Question,
};
use core::mem::size_of;

pub trait Section {}
pub struct QuestionsSection;
impl Section for QuestionsSection {}
pub struct AnswersSection;
impl Section for AnswersSection {}

pub struct MessageBody<
    'a,                 // Lifetime of the buffer
    B: Buffer + ?Sized, // Buffer type
    S: Section,         // The current section we are reading or writing to
    const WRITE: bool,  // Whether the buffer is mutable
> {
    _phantom: core::marker::PhantomData<(&'a B, S)>,
    header: *const Header,
    buffer: *const B,
    position: usize,
    question_count: u16,
    answer_count: u16,
}

impl<'a, B: Buffer + ?Sized, const WRITE: bool> MessageBody<'a, B, QuestionsSection, WRITE> {
    #[inline(always)]
    pub(crate) unsafe fn new(header: *const Header, buffer: *const B) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            header,
            buffer,
            position: size_of::<Header>(),
            question_count: unsafe { &*header }.question_count(),
            answer_count: unsafe { &*header }.answer_count(),
        }
    }

    #[inline(always)]
    pub fn questions(&mut self) -> Questions {
        Questions {
            buffer: unsafe { &*self.buffer }.bytes(),
            position: &mut self.position,
            count: &mut self.question_count,
        }
    }

    #[inline(always)]
    pub fn to_answer_section(mut self) -> MessageBody<'a, B, AnswersSection, WRITE> {
        let _ = self.questions().count();
        MessageBody {
            _phantom: core::marker::PhantomData,
            header: self.header,
            buffer: self.buffer,
            position: self.position,
            question_count: self.question_count,
            answer_count: self.answer_count,
        }
    }
}

impl<'a, B: ExtendableBuffer + ?Sized> MessageBody<'a, B, QuestionsSection, true> {
    pub fn append_question(
        &mut self,
        name: &[NameLabel],
        qtype: QType,
        qclass: QClass,
        move_to_end: bool,
    ) -> Result<(), ()> {
        if name.is_empty() {
            return Err(());
        }
        if self.question_count > 0 {
            // If we already have questions, we need to move the position to the end of the
            // questions section.
            let _ = self.questions().count();
        }
        let buffer = unsafe { &mut *(self.buffer as *mut B) };
        buffer.truncate(self.position); // Truncate the buffer to the end of the questions section

        for label in name {
            label.to_buffer(buffer)?;
        }
        // Add the null label if the last label is not a pointer.
        if !name.last().unwrap().is_pointer() {
            buffer.extend_from_slice(&[0])?;
        }
        // Add the question type.
        let qtype: u16 = qtype.into();
        buffer.extend_from_slice(&qtype.to_be_bytes())?;
        // Add the question class.
        let qclass: u16 = qclass.into();
        buffer.extend_from_slice(&qclass.to_be_bytes())?;
        // Update the header.
        let header = unsafe { &mut *(self.header as *mut Header) };
        let last_question_count = header.question_count();
        header.set_question_count(last_question_count + 1);
        // Since we added a question, the upcoming sections are overriden, hence we need to reset
        // the answer count.
        header.set_answer_count(0);
        self.answer_count = 0;
        if move_to_end {
            self.position = buffer.len();
        } else {
            self.question_count = 1;
        }

        Ok(())
    }
}

impl<'a, B: Buffer + ?Sized, const WRITE: bool> MessageBody<'a, B, AnswersSection, WRITE> {
    #[inline(always)]
    pub fn answers(&mut self) -> Answers {
        Answers {
            buffer: unsafe { &*self.buffer }.bytes(),
            position: &mut self.position,
            count: &mut self.answer_count,
        }
    }
}

impl<'a, B: ExtendableBuffer + ?Sized> MessageBody<'a, B, AnswersSection, true> {
    pub fn append_answer(
        &mut self,
        name: &[NameLabel],
        atype: AType<&[NameLabel]>,
        cache_flush: bool,
        aclass: AClass,
        ttl: u32,
        move_to_end: bool,
    ) -> Result<(), ()> {
        if name.is_empty() {
            return Err(());
        }
        if self.answer_count > 0 {
            // If we already have answers, we need to move the position to the end of the
            // answers section.
            let _ = self.answers().count();
        }
        let buffer = unsafe { &mut *(self.buffer as *mut B) };
        buffer.truncate(self.position); // Truncate the buffer to the end of the answers section

        for label in name {
            label.to_buffer(buffer)?;
        }
        // Add the null label if the last label is not a pointer.
        if !name.last().unwrap().is_pointer() {
            buffer.extend_from_slice(&[0])?;
        }
        // Add the answer type.
        let atype_id = atype.type_id()?;
        let mut atype_id = atype_id.to_be_bytes();
        atype_id[0] |= if cache_flush { 0b1000_0000 } else { 0 };
        buffer.extend_from_slice(&atype_id)?;
        // Add the answer class.
        let aclass: u16 = aclass.into();
        buffer.extend_from_slice(&aclass.to_be_bytes())?;
        // Add the answer ttl.
        buffer.extend_from_slice(&ttl.to_be_bytes())?;
        // Add the answer data.
        atype.to_buffer(buffer)?;

        // Update the header.
        let header = unsafe { &mut *(self.header as *mut Header) };
        let last_answer_count = header.answer_count();
        header.set_answer_count(last_answer_count + 1);
        if move_to_end {
            // Move the position to the end of the buffer.
            self.position = buffer.len();
        } else {
            self.answer_count = 1;
        }

        Ok(())
    }
}

pub struct Questions<'a> {
    buffer: &'a [u8],
    position: &'a mut usize,
    count: &'a mut u16,
}

impl<'a> Iterator for Questions<'a> {
    type Item = Question<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if *self.count == 0 {
            return None;
        }

        let question = Question::parse(self.buffer, self.position).ok()?;
        *self.count -= 1;

        Some(question)
    }
}

pub struct Answers<'a> {
    buffer: &'a [u8],
    position: &'a mut usize,
    count: &'a mut u16,
}

impl<'a> Iterator for Answers<'a> {
    type Item = Answer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if *self.count == 0 {
            return None;
        }

        let answer = Answer::parse(self.buffer, self.position).ok()?;
        *self.count -= 1;

        Some(answer)
    }
}
