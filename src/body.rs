use crate::{Answer, Buffer, ExtendableBuffer, Header, Question, QuestionBuilder, AnswerBuilder};
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
    pub(crate) header: *const Header,
    pub(crate) buffer: *const B,
    pub(crate) position: usize,
    pub(crate) question_count: u16,
    pub(crate) answer_count: u16,
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
    #[inline(always)]
    pub fn append_question(self) -> QuestionBuilder<
        'a,
        B,
        MessageBody<'a, B, QuestionsSection, true>,
        Result<MessageBody<'a, B, QuestionsSection, true>, ()>,
        fn(MessageBody<'a, B, QuestionsSection, true>) -> Result<MessageBody<'a, B, QuestionsSection, true>, ()>,
        false,
        false,
        false,
    > {
        QuestionBuilder::new(
            unsafe { &mut *(self.buffer as *mut B) },
            self,
            |parent| {
                let header = unsafe { &mut *(parent.header as *mut Header) };
                header.set_question_count(header.question_count() + 1);

                Ok(parent)
            },
        )
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
    #[inline(always)]
    pub fn append_answer(self) -> AnswerBuilder<
        'a,
        B,
        MessageBody<'a, B, AnswersSection, true>,
        Result<MessageBody<'a, B, AnswersSection, true>, ()>,
        fn(MessageBody<'a, B, AnswersSection, true>) -> Result<MessageBody<'a, B, AnswersSection, true>, ()>,
        false,
        false,
        false,
        false,
        false,
    > {
        AnswerBuilder::new(
            unsafe { &mut *(self.buffer as *mut B) },
            self,
            |parent| {
                let header = unsafe { &mut *(parent.header as *mut Header) };
                header.set_answer_count(header.answer_count() + 1);

                Ok(parent)
            },
        )
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
