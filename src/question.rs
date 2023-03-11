use crate::Name;
use derive_more::Display;

#[derive(Debug, PartialEq)]
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

    #[inline(always)]
    pub fn name(&self) -> &Name {
        &self.name
    }

    #[inline(always)]
    pub fn qtype(&self) -> QType {
        self.qtype
    }

    #[inline(always)]
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
    #[inline(always)]
    fn from(q: QType) -> Self {
        match q {
            QType::Reserved => panic!("Reserved QType"),
            _ => q as u16,
        }
    }
}

impl From<u16> for QType {
    #[inline(always)]
    fn from(n: u16) -> Self {
        match n {
            1..=16 | 252..=255 => unsafe { core::mem::transmute(n) },
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
    #[inline(always)]
    fn from(q: QClass) -> Self {
        match q {
            QClass::Reserved => panic!("Reserved QClass"),
            _ => q as u16,
        }
    }
}

impl From<u16> for QClass {
    #[inline(always)]
    fn from(n: u16) -> Self {
        match n {
            1..=4 => unsafe { core::mem::transmute(n) },
            _ => QClass::Reserved,
        }
    }
}
