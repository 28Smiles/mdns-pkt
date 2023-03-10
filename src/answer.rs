use crate::Name;
use derive_more::Display;

#[derive(Copy, Clone, Debug, Display, PartialEq)]
#[repr(u16)]
pub enum AClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
    Unknown,
}

impl From<u16> for AClass {
    fn from(n: u16) -> Self {
        match n {
            1 => AClass::IN,
            2 => AClass::CS,
            3 => AClass::CH,
            4 => AClass::HS,
            _ => AClass::Unknown,
        }
    }
}

impl From<AClass> for u16 {
    fn from(a: AClass) -> Self {
        match a {
            AClass::IN => 1,
            AClass::CS => 2,
            AClass::CH => 3,
            AClass::HS => 4,
            AClass::Unknown => panic!("Unknown AClass"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AType<'a> {
    /// A host address
    A(u32),
    /// An authoritative name server
    NS(Name<'a>),
    /// The canonical name for an alias
    CNAME(Name<'a>),
    /// Marks the start of a zone of authority
    SOA(Name<'a>, Name<'a>, u32, u32, u32, u32, u32),
    /// A domain name pointer
    PTR(Name<'a>),
    /// Mail exchange
    MX(u16, Name<'a>),
    /// Text strings
    TXT(&'a [u8]),
    /// IPv6 address
    AAAA([u8; 16]),
    /// Location information
    SRV(u16, u16, u16, Name<'a>),
    /// OPT pseudo-RR
    OPT(u16, u8, u8, &'a [u8]),
    /// Unknown
    Unknown,
}
impl<'a> AType<'a> {
    pub fn type_id(&self) -> Result<u16, ()> {
        match self {
            AType::A(_) => Ok(1),
            AType::NS(_) => Ok(2),
            AType::CNAME(_) => Ok(5),
            AType::SOA(_, _, _, _, _, _, _) => Ok(6),
            AType::PTR(_) => Ok(12),
            AType::MX(_, _) => Ok(15),
            AType::TXT(_) => Ok(16),
            AType::AAAA(_) => Ok(28),
            AType::SRV(_, _, _, _) => Ok(33),
            AType::OPT(_, _, _, _) => Ok(41),
            AType::Unknown => Err(()),
        }
    }

    pub fn parse(atype: u16, data_len: u16, bytes: &'a [u8], i: &mut usize) -> Result<Self, ()> {
        match atype {
            1 => {
                if data_len != 4 {
                    return Err(());
                }
                let addr =
                    u32::from_be_bytes([bytes[*i], bytes[*i + 1], bytes[*i + 2], bytes[*i + 3]]);
                *i += 4;
                Ok(AType::A(addr))
            }
            2 => {
                let mut j = *i;
                let name = Name::parse(bytes, &mut j)?;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::NS(name))
            }
            5 => {
                let mut j = *i;
                let name = Name::parse(bytes, &mut j)?;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::CNAME(name))
            }
            6 => {
                let mut j = *i;
                let mname = Name::parse(bytes, &mut j)?;
                let rname = Name::parse(bytes, &mut j)?;
                let serial =
                    u32::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
                j += 4;
                let refresh =
                    u32::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
                j += 4;
                let retry =
                    u32::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
                j += 4;
                let expire =
                    u32::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
                j += 4;
                let minimum =
                    u32::from_be_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
                j += 4;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::SOA(
                    mname, rname, serial, refresh, retry, expire, minimum,
                ))
            }
            12 => {
                let mut j = *i;
                let name = Name::parse(bytes, &mut j)?;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::PTR(name))
            }
            15 => {
                let mut j = *i;
                let preference = u16::from_be_bytes([bytes[j], bytes[j + 1]]);
                j += 2;
                let exchange = Name::parse(bytes, &mut j)?;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::MX(preference, exchange))
            }
            16 => {
                if data_len < 1 {
                    return Err(());
                }
                let txt = &bytes[*i..*i + data_len as usize];
                *i += data_len as usize;

                Ok(AType::TXT(txt))
            }
            28 => {
                if data_len != 16 {
                    return Err(());
                }
                let mut addr = [0; 16];
                addr.copy_from_slice(&bytes[*i..*i + 16]);
                *i += 16;
                Ok(AType::AAAA(addr))
            }
            33 => {
                let mut j = *i;
                let priority = u16::from_be_bytes([bytes[j], bytes[j + 1]]);
                j += 2;
                let weight = u16::from_be_bytes([bytes[j], bytes[j + 1]]);
                j += 2;
                let port = u16::from_be_bytes([bytes[j], bytes[j + 1]]);
                j += 2;
                let target = Name::parse(bytes, &mut j)?;
                if j - *i != data_len as usize {
                    return Err(());
                }
                *i = j;

                Ok(AType::SRV(priority, weight, port, target))
            }
            41 => {
                if data_len < 4 {
                    return Err(());
                }
                let udp_payload_size = u16::from_be_bytes([bytes[*i], bytes[*i + 1]]);
                *i += 2;
                let extended_rcode = bytes[*i];
                *i += 1;
                let version = bytes[*i];
                *i += 1;
                let data_len = data_len as usize - 4;
                let data = &bytes[*i..*i + data_len];
                *i += data_len;

                Ok(AType::OPT(udp_payload_size, extended_rcode, version, data))
            }
            _ => {
                *i += data_len as usize;
                Ok(AType::Unknown)
            }
        }
    }
}

pub struct Answer<'a> {
    name: Name<'a>,
    atype: AType<'a>,
    cache_flush: bool,
    aclass: AClass,
    ttl: u32,
}

impl<'a> Answer<'a> {
    pub fn parse(bytes: &'a [u8], i: &mut usize) -> Result<Self, ()> {
        let name = Name::parse(bytes, i)?;
        let atype = u16::from_be_bytes([bytes[*i], bytes[*i + 1]]);
        *i += 2;
        let cache_flush = (bytes[*i] & 0b1000_0000) != 0;
        let aclass = AClass::from(u16::from_be_bytes([bytes[*i] & 0b0111_1111, bytes[*i + 1]]));
        *i += 2;
        let ttl = u32::from_be_bytes([bytes[*i], bytes[*i + 1], bytes[*i + 2], bytes[*i + 3]]);
        *i += 4;
        let data_len = u16::from_be_bytes([bytes[*i], bytes[*i + 1]]);
        *i += 2;
        let atype = AType::parse(atype, data_len, bytes, i)?;

        Ok(Answer {
            name,
            atype,
            cache_flush,
            aclass,
            ttl,
        })
    }

    #[inline(always)]
    pub fn name(&self) -> &Name<'a> {
        &self.name
    }

    #[inline(always)]
    pub fn atype(&self) -> &AType<'a> {
        &self.atype
    }

    #[inline(always)]
    pub fn cache_flush(&self) -> bool {
        self.cache_flush
    }

    #[inline(always)]
    pub fn aclass(&self) -> AClass {
        self.aclass
    }

    #[inline(always)]
    pub fn ttl(&self) -> u32 {
        self.ttl
    }
}
