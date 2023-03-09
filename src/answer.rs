use crate::{ExtendableBuffer, Name, NameLabel};
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
pub enum AType<'a, NameType> {
    /// A host address
    A(u32),
    /// An authoritative name server
    NS(NameType),
    /// The canonical name for an alias
    CNAME(NameType),
    /// Marks the start of a zone of authority
    SOA(NameType, NameType, u32, u32, u32, u32, u32),
    /// A domain name pointer
    PTR(NameType),
    /// Mail exchange
    MX(u16, NameType),
    /// Text strings
    TXT(&'a [u8]),
    /// IPv6 address
    AAAA([u8; 16]),
    /// Location information
    SRV(u16, u16, u16, NameType),
    /// OPT pseudo-RR
    OPT(u16, u8, u8, &'a [u8]),
    /// Unknown
    Unknown,
}
impl<'a, NameType> AType<'a, NameType> {
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
}

impl<'a> AType<'a, Name<'a>> {
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

impl<'a> AType<'a, &'a [NameLabel<'a>]> {
    #[inline(always)]
    pub(crate) fn to_buffer<B: ExtendableBuffer + ?Sized>(&self, buffer: &mut B) -> Result<(), ()> {
        match self {
            AType::A(addr) => {
                // Write the length of the data.
                buffer.extend_from_slice(&4u16.to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(&addr.to_be_bytes())?;

                Ok(())
            }
            AType::NS(name) | AType::CNAME(name) => {
                // Write the length of the data.
                let mut len = name.iter().map(|label| label.len()).sum::<usize>();
                if !name.last().unwrap().is_pointer() {
                    len += 1;
                }
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                for label in name.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !name.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }

                Ok(())
            }
            AType::SOA(mname, rname, serial, refresh, retry, expire, minimum) => {
                // Write the length of the data.
                let mut len = mname.iter().map(|label| label.len()).sum::<usize>();
                if !mname.last().unwrap().is_pointer() {
                    len += 1;
                }
                len += rname.iter().map(|label| label.len()).sum::<usize>();
                if !rname.last().unwrap().is_pointer() {
                    len += 1;
                }
                len += 20;
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                for label in mname.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !mname.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }
                for label in rname.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !rname.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }
                buffer.extend_from_slice(&serial.to_be_bytes())?;
                buffer.extend_from_slice(&refresh.to_be_bytes())?;
                buffer.extend_from_slice(&retry.to_be_bytes())?;
                buffer.extend_from_slice(&expire.to_be_bytes())?;
                buffer.extend_from_slice(&minimum.to_be_bytes())?;

                Ok(())
            }
            AType::PTR(name) => {
                // Write the length of the data.
                let mut len = name.iter().map(|label| label.len()).sum::<usize>();
                if !name.last().unwrap().is_pointer() {
                    len += 1;
                }
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                for label in name.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !name.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }

                Ok(())
            }
            AType::MX(preference, name) => {
                // Write the length of the data.
                let mut len = 2;
                len += name.iter().map(|label| label.len()).sum::<usize>();
                if !name.last().unwrap().is_pointer() {
                    len += 1;
                }
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(&preference.to_be_bytes())?;
                for label in name.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !name.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }

                Ok(())
            }
            AType::TXT(data) => {
                // Write the length of the data.
                buffer.extend_from_slice(&(data.len() as u16).to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(*data)?;

                Ok(())
            }
            AType::AAAA(addr) => {
                // Write the length of the data.
                buffer.extend_from_slice(&16u16.to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(addr)?;

                Ok(())
            }
            AType::SRV(priority, weight, port, name) => {
                // Write the length of the data.
                let mut len = 6;
                len += name.iter().map(|label| label.len()).sum::<usize>();
                if !name.last().unwrap().is_pointer() {
                    len += 1;
                }
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(&priority.to_be_bytes())?;
                buffer.extend_from_slice(&weight.to_be_bytes())?;
                buffer.extend_from_slice(&port.to_be_bytes())?;
                for label in name.iter() {
                    buffer.extend_from_slice(&label.len().to_be_bytes())?;
                    label.to_buffer(buffer)?;
                }
                if !name.last().unwrap().is_pointer() {
                    buffer.extend_from_slice(&0u8.to_be_bytes())?;
                }

                Ok(())
            }
            AType::OPT(udp_payload_size, extended_rcode, version, data) => {
                // Write the length of the data.
                let mut len = 4;
                len += data.len();
                buffer.extend_from_slice(&(len as u16).to_be_bytes())?;
                // Write the data.
                buffer.extend_from_slice(&udp_payload_size.to_be_bytes())?;
                buffer.extend_from_slice(&((extended_rcode << 4) | version).to_be_bytes())?;
                buffer.extend_from_slice(*data)?;

                Ok(())
            }
            AType::Unknown => Err(()),
        }
    }
}

pub struct Answer<'a> {
    name: Name<'a>,
    atype: AType<'a, Name<'a>>,
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
    pub fn atype(&self) -> &AType<'a, Name<'a>> {
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
