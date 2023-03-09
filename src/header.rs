use derive_more::Display;

/// A mDNS header.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Header {
    id: [u8; 2],
    flags: [u8; 2],
    question_count: [u8; 2],
    answer_count: [u8; 2],
    name_server_count: [u8; 2],
    additional_records_count: [u8; 2],
}

impl Header {
    #[inline(always)]
    pub fn from_bytes(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) }
    }

    #[inline(always)]
    pub fn new(
        id: u16,
        kind: HeaderKind,
        opcode: HeaderOpcode,
        authoritative_answer: bool,
        truncated_message: bool,
        recursion_desired: bool,
        recursion_available: bool,
        response_code: HeaderResponseCode,
    ) -> Self {
        let mut flags = [0; 2];
        flags[0] = if kind == HeaderKind::Response {
            0b1000_0000
        } else {
            0b0000_0000
        };

        let opcode: u8 = opcode.into();
        flags[0] |= (opcode << 3) & 0b0111_1000;
        flags[0] |= if authoritative_answer {
            0b0000_0100
        } else {
            0b0000_0000
        };
        flags[0] |= if truncated_message {
            0b0000_0010
        } else {
            0b0000_0000
        };
        flags[0] |= if recursion_desired {
            0b0000_0001
        } else {
            0b0000_0000
        };
        flags[1] |= if recursion_available {
            0b1000_0000
        } else {
            0b0000_0000
        };
        let response_code: u8 = response_code.into();
        flags[1] |= response_code & 0b0000_1111;

        Self {
            id: id.to_be_bytes(),
            flags,
            question_count: [0; 2],
            answer_count: [0; 2],
            name_server_count: [0; 2],
            additional_records_count: [0; 2],
        }
    }

    #[inline(always)]
    pub fn id(&self) -> u16 {
        u16::from_be_bytes(self.id)
    }

    #[inline(always)]
    pub fn kind(&self) -> HeaderKind {
        if (self.flags[0] & 0b10000000) == 0 {
            HeaderKind::Query
        } else {
            HeaderKind::Response
        }
    }

    #[inline(always)]
    pub fn opcode(&self) -> HeaderOpcode {
        (self.flags[0] & 0b01111000).into()
    }

    #[inline(always)]
    pub fn authoritative_answer(&self) -> bool {
        (self.flags[0] & 0b00000100) != 0
    }

    #[inline(always)]
    pub fn truncated(&self) -> bool {
        (self.flags[0] & 0b00000010) != 0
    }

    #[inline(always)]
    pub fn recursion_desired(&self) -> bool {
        (self.flags[0] & 0b00000001) != 0
    }

    #[inline(always)]
    pub fn recursion_available(&self) -> bool {
        (self.flags[1] & 0b10000000) != 0
    }

    #[inline(always)]
    pub fn response_code(&self) -> HeaderResponseCode {
        (self.flags[1] & 0b00001111).into()
    }

    #[inline(always)]
    pub fn question_count(&self) -> u16 {
        u16::from_be_bytes(self.question_count)
    }

    #[inline(always)]
    pub fn answer_count(&self) -> u16 {
        u16::from_be_bytes(self.answer_count)
    }

    #[inline(always)]
    pub fn name_server_count(&self) -> u16 {
        u16::from_be_bytes(self.name_server_count)
    }

    #[inline(always)]
    pub fn additional_records_count(&self) -> u16 {
        u16::from_be_bytes(self.additional_records_count)
    }

    #[inline(always)]
    pub fn set_id(&mut self, id: u16) {
        self.id = id.to_be_bytes();
    }

    #[inline(always)]
    pub fn set_kind(&mut self, kind: HeaderKind) {
        match kind {
            HeaderKind::Query => self.flags[0] &= 0b01111111,
            HeaderKind::Response => self.flags[0] |= 0b10000000,
        }
    }

    #[inline(always)]
    pub fn set_opcode(&mut self, opcode: HeaderOpcode) {
        self.flags[0] &= 0b10000111;
        self.flags[0] |= (u8::from(opcode) & 0b0000_1111) << 3;
    }

    #[inline(always)]
    pub fn set_authoritative_answer(&mut self, authoritative_answer: bool) {
        if authoritative_answer {
            self.flags[0] |= 0b00000100;
        } else {
            self.flags[0] &= 0b11111011;
        }
    }

    #[inline(always)]
    pub fn set_truncated(&mut self, truncated: bool) {
        if truncated {
            self.flags[0] |= 0b00000010;
        } else {
            self.flags[0] &= 0b11111101;
        }
    }

    #[inline(always)]
    pub fn set_recursion_desired(&mut self, recursion_desired: bool) {
        if recursion_desired {
            self.flags[0] |= 0b00000001;
        } else {
            self.flags[0] &= 0b11111110;
        }
    }

    #[inline(always)]
    pub fn set_recursion_available(&mut self, recursion_available: bool) {
        if recursion_available {
            self.flags[1] |= 0b10000000;
        } else {
            self.flags[1] &= 0b01111111;
        }
    }

    #[inline(always)]
    pub fn set_response_code(&mut self, response_code: HeaderResponseCode) {
        self.flags[1] &= 0b11110000;
        self.flags[1] |= u8::from(response_code) & 0b00001111;
    }

    #[inline(always)]
    pub(crate) fn set_question_count(&mut self, question_count: u16) {
        self.question_count = question_count.to_be_bytes();
    }

    #[inline(always)]
    pub(crate) fn set_answer_count(&mut self, answer_count: u16) {
        self.answer_count = answer_count.to_be_bytes();
    }

    #[inline(always)]
    pub(crate) fn set_name_server_count(&mut self, name_server_count: u16) {
        self.name_server_count = name_server_count.to_be_bytes();
    }

    #[inline(always)]
    pub(crate) fn set_additional_records_count(&mut self, additional_records_count: u16) {
        self.additional_records_count = additional_records_count.to_be_bytes();
    }
}

/// The kind of a DNS header.
#[derive(Copy, Clone, Debug, Display, PartialEq)]
pub enum HeaderKind {
    Query,
    Response,
}

/// A DNS opcode.
#[derive(Copy, Clone, Debug, Display, PartialEq)]
pub enum HeaderOpcode {
    Query,
    InverseQuery,
    Status,
    Notify,
    Update,
    Reserved(u8),
}

impl From<u8> for HeaderOpcode {
    fn from(value: u8) -> Self {
        match value {
            0 => HeaderOpcode::Query,
            1 => HeaderOpcode::InverseQuery,
            2 => HeaderOpcode::Status,
            4 => HeaderOpcode::Notify,
            5 => HeaderOpcode::Update,
            _ => HeaderOpcode::Reserved(value),
        }
    }
}

impl From<HeaderOpcode> for u8 {
    fn from(value: HeaderOpcode) -> Self {
        match value {
            HeaderOpcode::Query => 0,
            HeaderOpcode::InverseQuery => 1,
            HeaderOpcode::Status => 2,
            HeaderOpcode::Notify => 4,
            HeaderOpcode::Update => 5,
            HeaderOpcode::Reserved(value) => value,
        }
    }
}

/// A DNS response code.
#[derive(Copy, Clone, Debug, Display, PartialEq)]
pub enum HeaderResponseCode {
    NoError,
    FormatError,
    ServerFailure,
    NonExistentDomain,
    NotImplemented,
    Refused,
    ExistentDomain,
    ExistentRrSet,
    NonExistentRrSet,
    NotAuthoritative,
    NotZone,
    BadOptVersionOrBadSignature,
    BadKey,
    BadTime,
    BadMode,
    BadName,
    BadAlg,
    Reserved(u8),
}

impl From<HeaderResponseCode> for u8 {
    fn from(r: HeaderResponseCode) -> Self {
        match r {
            HeaderResponseCode::NoError => 0,
            HeaderResponseCode::FormatError => 1,
            HeaderResponseCode::ServerFailure => 2,
            HeaderResponseCode::NonExistentDomain => 3,
            HeaderResponseCode::NotImplemented => 4,
            HeaderResponseCode::Refused => 5,
            HeaderResponseCode::ExistentDomain => 6,
            HeaderResponseCode::ExistentRrSet => 7,
            HeaderResponseCode::NonExistentRrSet => 8,
            HeaderResponseCode::NotAuthoritative => 9,
            HeaderResponseCode::NotZone => 10,
            HeaderResponseCode::BadOptVersionOrBadSignature => 16,
            HeaderResponseCode::BadKey => 17,
            HeaderResponseCode::BadTime => 18,
            HeaderResponseCode::BadMode => 19,
            HeaderResponseCode::BadName => 20,
            HeaderResponseCode::BadAlg => 21,
            HeaderResponseCode::Reserved(n) => n,
        }
    }
}

impl From<u8> for HeaderResponseCode {
    fn from(n: u8) -> Self {
        match n {
            0 => HeaderResponseCode::NoError,
            1 => HeaderResponseCode::FormatError,
            2 => HeaderResponseCode::ServerFailure,
            3 => HeaderResponseCode::NonExistentDomain,
            4 => HeaderResponseCode::NotImplemented,
            5 => HeaderResponseCode::Refused,
            6 => HeaderResponseCode::ExistentDomain,
            7 => HeaderResponseCode::ExistentRrSet,
            8 => HeaderResponseCode::NonExistentRrSet,
            9 => HeaderResponseCode::NotAuthoritative,
            10 => HeaderResponseCode::NotZone,
            16 => HeaderResponseCode::BadOptVersionOrBadSignature,
            17 => HeaderResponseCode::BadKey,
            18 => HeaderResponseCode::BadTime,
            19 => HeaderResponseCode::BadMode,
            20 => HeaderResponseCode::BadName,
            21 => HeaderResponseCode::BadAlg,
            n => HeaderResponseCode::Reserved(n),
        }
    }
}
