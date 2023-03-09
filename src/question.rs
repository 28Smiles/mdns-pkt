use crate::Name;
use derive_more::Display;

pub struct Question<'a> {
    name: Name<'a>,
    qtype: QType,
    qclass: QClass,
}

impl<'a> Question<'a> {
    pub fn parse(bytes: &'a [u8], i: &mut usize) -> Result<Self, ()> {
        let name = Name::parse(bytes, i)?;
        let qtype = u16::from_be_bytes([bytes[*i], bytes[*i + 1]]).into();
        let qclass = u16::from_be_bytes([bytes[*i + 2], bytes[*i + 3]]).into();
        *i += 4;

        Ok(Self {
            name,
            qtype,
            qclass,
        })
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn qtype(&self) -> QType {
        self.qtype
    }

    pub fn qclass(&self) -> QClass {
        self.qclass
    }
}

/// The kind of a DNS query.
///
/// According to [RFC 1035 Section 3.2.2](https://tools.ietf.org/rfc/rfc1035#section-3.2.2)
/// and [RFC 1035 Section 3.2.3](https://tools.ietf.org/rfc/rfc1035#section-3.2.3).
#[derive(Copy, Clone, Debug, Display, PartialEq)]
#[repr(u16)]
pub enum QType {
    A = 1,
    NS = 2,
    MD = 3,
    MF = 4,
    CNAME = 5,
    SOA = 6,
    MB = 7,
    MG = 8,
    MR = 9,
    NULL = 10,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
    AXFR = 252,
    MAILB = 253,
    MAILA = 254,
    ALL = 255,
    Reserved,
}

impl From<QType> for u16 {
    fn from(q: QType) -> Self {
        match q {
            QType::A => 1,
            QType::NS => 2,
            QType::MD => 3,
            QType::MF => 4,
            QType::CNAME => 5,
            QType::SOA => 6,
            QType::MB => 7,
            QType::MG => 8,
            QType::MR => 9,
            QType::NULL => 10,
            QType::WKS => 11,
            QType::PTR => 12,
            QType::HINFO => 13,
            QType::MINFO => 14,
            QType::MX => 15,
            QType::TXT => 16,
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::ALL => 255,
            QType::Reserved => panic!("Reserved QType"),
        }
    }
}

impl From<u16> for QType {
    fn from(n: u16) -> Self {
        match n {
            1 => QType::A,
            2 => QType::NS,
            3 => QType::MD,
            4 => QType::MF,
            5 => QType::CNAME,
            6 => QType::SOA,
            7 => QType::MB,
            8 => QType::MG,
            9 => QType::MR,
            10 => QType::NULL,
            11 => QType::WKS,
            12 => QType::PTR,
            13 => QType::HINFO,
            14 => QType::MINFO,
            15 => QType::MX,
            16 => QType::TXT,
            252 => QType::AXFR,
            253 => QType::MAILB,
            254 => QType::MAILA,
            255 => QType::ALL,
            _ => QType::Reserved,
        }
    }
}

/// The class of a DNS query.
///
/// According to [RFC 1035 Section 3.2.4](https://tools.ietf.org/rfc/rfc1035#section-3.2.4).
#[derive(Copy, Clone, Debug, Display, PartialEq)]
#[repr(u16)]
pub enum QClass {
    /// Internet
    IN = 1,
    /// CSNET
    CS = 2,
    /// CHAOS
    CH = 3,
    /// Hesiod
    HS = 4,
    Reserved,
}

impl From<QClass> for u16 {
    fn from(q: QClass) -> Self {
        match q {
            QClass::IN => 1,
            QClass::CS => 2,
            QClass::CH => 3,
            QClass::HS => 4,
            QClass::Reserved => panic!("Reserved QClass"),
        }
    }
}

impl From<u16> for QClass {
    fn from(n: u16) -> Self {
        match n {
            1 => QClass::IN,
            2 => QClass::CS,
            3 => QClass::CH,
            4 => QClass::HS,
            _ => QClass::Reserved,
        }
    }
}
