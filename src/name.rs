use crate::ExtendableBuffer;

/// A DNS name.
#[derive(Debug, Clone, PartialEq)]
pub struct Name<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Name<'a> {
    pub fn parse(bytes: &'a [u8], i: &mut usize) -> Result<Self, ()> {
        const MAX_LENGTH: usize = 255;
        let mut j = *i;

        loop {
            if j - *i >= MAX_LENGTH {
                return Err(());
            }

            match LabelType::from_bytes(bytes, &mut j)? {
                LabelType::Pointer(_) => {
                    break;
                }
                LabelType::Part(len) => {
                    j += len as usize;

                    if len == 0 {
                        break;
                    }
                }
            }
        }

        let offset = *i;
        *i = j;

        Ok(Self { bytes, offset })
    }
}

enum LabelType {
    Pointer(u16),
    Part(u8),
}

impl LabelType {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self, ()> {
        const PTR_MASK: u8 = 0b11000000;
        const LEN_MASK: u8 = !PTR_MASK;

        let c = bytes[*i];

        if c & PTR_MASK == PTR_MASK {
            let pointer = u16::from_be_bytes([c, bytes[1]]);
            if pointer > *i as u16 {
                // Cannot point to the future.
                return Err(());
            }

            *i += 2;
            Ok(Self::Pointer(pointer))
        } else {
            let len = c & LEN_MASK;
            *i += 1;

            Ok(Self::Part(len))
        }
    }
}

#[derive(Clone)]
enum NameLabelType<'a> {
    Pointer(u16),
    Part(&'a [u8]),
}

#[derive(Clone)]
pub struct NameLabel<'a> {
    inner: NameLabelType<'a>,
}

impl<'a> NameLabel<'a> {
    #[inline(always)]
    pub const fn new_part(bytes: &'a [u8]) -> Self {
        Self {
            inner: NameLabelType::Part(bytes),
        }
    }

    #[inline(always)]
    pub const fn new_pointer(to: &Name) -> Self {
        Self {
            inner: NameLabelType::Pointer(to.offset as u16),
        }
    }

    #[inline(always)]
    pub(crate) fn to_buffer<B: ExtendableBuffer + ?Sized>(&self, buffer: &mut B) -> Result<(), ()> {
        match &self.inner {
            NameLabelType::Pointer(ptr) => {
                let mut bytes = ptr.to_be_bytes();
                bytes[0] |= 0b11000000;
                buffer.extend_from_slice(&bytes)?;
                Ok(())
            }
            NameLabelType::Part(bytes) => {
                buffer.extend_from_slice(&[bytes.len() as u8])?;
                buffer.extend_from_slice(bytes)?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    pub fn is_pointer(&self) -> bool {
        matches!(self.inner, NameLabelType::Pointer(_))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.inner {
            NameLabelType::Pointer(_) => 2,
            NameLabelType::Part(bytes) => 1 + bytes.len(),
        }
    }
}