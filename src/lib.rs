#![no_std]

mod message;
mod header;
mod name;
mod question;
mod body;
mod answer;
mod builder;

pub use message::*;
pub use header::*;
pub use name::*;
pub use question::*;
pub use body::*;
pub use answer::*;
pub use builder::*;

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use super::*;

    #[test]
    fn mdns_parse_question() {
        let data: &[u8] = &[
            0x00, 0x08, // ID
            0b0000_0001, 0b0000_0000, // Flags
            0x00, 0x01, // Question count
            0x00, 0x00, // Answer count
            0x00, 0x00, // Name server count
            0x00, 0x00, // Additional records count
            // Question
            // Name
            0x08, 0x5f, 0x61, 0x69, 0x72, 0x70, 0x6f, 0x72, 0x74, // _airport
            0x04, 0x5f, 0x74, 0x63, 0x70, // _tcp
            0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, // local
            0x00, // Null terminator
            0x00, 0x01, // Type
            0x00, 0x01, // Class
            // The rest of the packet is ignored
            0xc0, 0x0c, 0x00, 0x0c, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x78, 0x00, 0x10,
            0x00, 0x00, 0x00, 0x78, 0x00, 0x10,
        ];

        let message = Message::new(data).unwrap();
        let header = message.header().unwrap();
        assert_eq!(header.id(), 8);
        assert_eq!(header.kind(), HeaderKind::Query);
        assert_eq!(header.opcode(), HeaderOpcode::Query);
        assert_eq!(header.authoritative_answer(), false);
        assert_eq!(header.truncated(), false);
        assert_eq!(header.recursion_desired(), true);
        assert_eq!(header.recursion_available(), false);
        assert_eq!(header.response_code(), HeaderResponseCode::NoError);
        assert_eq!(header.question_count(), 1);
        assert_eq!(header.answer_count(), 0);
        assert_eq!(header.name_server_count(), 0);
        assert_eq!(header.additional_records_count(), 0);
        let mut body = message.body().unwrap();
        let question = body.questions().next().unwrap();
        assert_eq!(question.name(), b"_airport._tcp.local".as_slice());
        assert_eq!(question.qtype(), QType::A);
        assert_eq!(question.qclass(), QClass::IN);

        let mut body = body.to_answer_section();
        assert_eq!(body.answers().next().is_none(), true);
    }

    #[test]
    fn mdns_write_question() {
        let data: &[u8] = &[
            0x00, 0x08, // ID
            0b0000_0001, 0b0000_0000, // Flags
            0x00, 0x01, // Question count
            0x00, 0x00, // Answer count
            0x00, 0x00, // Name server count
            0x00, 0x00, // Additional records count
            // Question
            // Name
            0x08, 0x5f, 0x61, 0x69, 0x72, 0x70, 0x6C, 0x61, 0x79, // _airport
            0x04, 0x5f, 0x74, 0x63, 0x70, // _tcp
            0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, // local
            0x00, // Null terminator
            0x00, 0x01, // Type
            0x00, 0x01, // Class
        ];

        let mut write_buffer = ArrayVec::<u8, 256>::new_const();
        let mut message = Message::new_mut(&mut write_buffer).unwrap();
        let header = message.header_mut().unwrap();
        header.set_id(8);
        header.set_kind(HeaderKind::Query);
        header.set_opcode(HeaderOpcode::Query);
        header.set_authoritative_answer(false);
        header.set_truncated(false);
        header.set_recursion_desired(true);
        header.set_recursion_available(false);
        header.set_response_code(HeaderResponseCode::NoError);
        let body = message.body_mut().unwrap();
        body.append_question()
            .name()
            .label(b"_airplay").unwrap()
            .label(b"_tcp").unwrap()
            .label(b"local").unwrap()
            .finish().unwrap()
            .qtype(QType::A).unwrap()
            .qclass(QClass::IN).unwrap()
            .finish().unwrap();

        assert_eq!(write_buffer.as_slice(), data);
    }

    #[test]
    fn mdns_parse_answer() {
        let data: &[u8] = &[
            0x00, 0x08, // ID
            0b0000_0001, 0b0000_0000, // Flags
            0x00, 0x00, // Question count
            0x00, 0x02, // Answer count
            0x00, 0x00, // Name server count
            0x00, 0x00, // Additional records count
            // Answer 0
            0x08, 0x5f, 0x61, 0x69, 0x72, 0x70, 0x6C, 0x61, 0x79, // _airport
            0x04, 0x5f, 0x74, 0x63, 0x70, // _tcp
            0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, // local
            0x00, // Null terminator
            0x00, 0x01, // Type
            0x00, 0x01, // Class
            0x00, 0x00, 0x00, 0x78, // TTL
            0x00, 0x04, // Length
            0xc0, 0xa8, 0x00, 0x01, // IP
            // Answer 1
            0x08, 0x5f, 0x61, 0x69, 0x72, 0x70, 0x6C, 0x61, 0x79, // _airport
            0x04, 0x5f, 0x74, 0x63, 0x70, // _tcp
            0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, // local
            0x00, // Null terminator
            0x00, 0x10, // Type (TXT)
            0x00, 0x01, // Class
            0x00, 0x00, 0x00, 0x78, // TTL
            0x00, 0x0e, // Length
            0x00, 0x08, // Length of first part
            0x6d, 0x64, 0x3d, 0x4d, 0x69, 0x6e, 0x69, 0x54, // md=MiniT
            0x00, 0x02, // Length of second part
            0x76, 0x3d, // v=1
        ];

        let message = Message::new(data).unwrap();
        let header = message.header().unwrap();
        assert_eq!(header.id(), 8);
        assert_eq!(header.kind(), HeaderKind::Query);
        assert_eq!(header.opcode(), HeaderOpcode::Query);
        assert_eq!(header.authoritative_answer(), false);
        assert_eq!(header.truncated(), false);
        assert_eq!(header.recursion_desired(), true);
        assert_eq!(header.recursion_available(), false);
        assert_eq!(header.response_code(), HeaderResponseCode::NoError);
        assert_eq!(header.question_count(), 0);
        assert_eq!(header.answer_count(), 2);
        assert_eq!(header.name_server_count(), 0);
        assert_eq!(header.additional_records_count(), 0);
        let body = message.body().unwrap();
        let mut body = body.to_answer_section();
        let mut answers = body.answers();
        let answer = answers.next().unwrap();
        assert_eq!(answer.name(), b"_airplay._tcp.local".as_slice());
        assert_eq!(answer.atype(), &AType::A(u32::from_be_bytes([192, 168, 0, 1])));
        assert_eq!(answer.cache_flush(), false);
        assert_eq!(answer.aclass(), AClass::IN);
        assert_eq!(answer.ttl(), 120);
        let answer = answers.next().unwrap();
        assert_eq!(answer.name(), b"_airplay._tcp.local".as_slice());
        assert_eq!(answer.atype(), &AType::TXT(&[
            0x00, 0x08,
            0x6d, 0x64, 0x3d, 0x4d, 0x69, 0x6e, 0x69, 0x54,
            0x00, 0x02,
            0x76, 0x3d
        ]));
        assert_eq!(answer.aclass(), AClass::IN);
        assert_eq!(answer.ttl(), 120);
    }

    #[test]
    fn mdns_write_answer() {
        let data: &[u8] = &[
            0x00, 0x08, // ID
            0b0000_0001, 0b0000_0000, // Flags
            0x00, 0x00, // Question count
            0x00, 0x02, // Answer count
            0x00, 0x00, // Name server count
            0x00, 0x00, // Additional records count
            // Answer 0
            0x08, 0x5f, 0x61, 0x69, 0x72, 0x70, 0x6C, 0x61, 0x79, // _airplay
            0x04, 0x5f, 0x74, 0x63, 0x70, // _tcp
            0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, // local
            0x00, // Null terminator
            0x00, 0x01, // Type
            0x00, 0x01, // Class
            0x00, 0x00, 0x00, 0x78, // TTL
            0x00, 0x04, // Length
            0xc0, 0xa8, 0x00, 0x01, // IP
            // Answer 1
            0b1100_0000, 0x0C, // Pointer to _airplay._tcp.local
            0x00, 0x10, // Type (TXT)
            0x00, 0x01, // Class
            0x00, 0x00, 0x00, 0x78, // TTL
            0x00, 0x0e, // Length
            0x00, 0x08, // Length of first part
            0x6d, 0x64, 0x3d, 0x4d, 0x69, 0x6e, 0x69, 0x54, // md=MiniT
            0x00, 0x02, // Length of second part
            0x76, 0x3d, // v=1
        ];

        let mut write_buffer = ArrayVec::<u8, 256>::new_const();
        let mut message = Message::new_mut(&mut write_buffer).unwrap();
        let header = message.header_mut().unwrap();
        header.set_id(8);
        header.set_kind(HeaderKind::Query);
        header.set_opcode(HeaderOpcode::Query);
        header.set_authoritative_answer(false);
        header.set_truncated(false);
        header.set_recursion_desired(true);
        header.set_recursion_available(false);
        header.set_response_code(HeaderResponseCode::NoError);
        let body = message.body_mut().unwrap();
        let body = body.to_answer_section();
        let body = body.append_answer().name();
        let name_ptr = body.ptr();
        let body = body
            .label(b"_airplay").unwrap()
            .label(b"_tcp").unwrap()
            .label(b"local").unwrap()
            .finish().unwrap().unwrap()
            .atype()
            .a()
            .ip(&[192, 168, 0, 1]).unwrap()
            .finish().unwrap()
            .cache_flush(false).unwrap()
            .aclass(AClass::IN).unwrap()
            .ttl(120).unwrap()
            .finish().unwrap();
        body.append_answer()
            .name()
            .label(&name_ptr).unwrap()
            .finish().unwrap().unwrap()
            .atype()
            .txt()
            .txt(&[
                0x00, 0x08,
                0x6d, 0x64, 0x3d, 0x4d, 0x69, 0x6e, 0x69, 0x54,
                0x00, 0x02,
                0x76, 0x3d
            ]).unwrap()
            .finish().unwrap()
            .cache_flush(false).unwrap()
            .aclass(AClass::IN).unwrap()
            .ttl(120).unwrap()
            .finish().unwrap();

        assert_eq!(write_buffer.as_slice(), data);
    }
}
