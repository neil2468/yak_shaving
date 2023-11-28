use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(PartialEq, Debug)]
pub enum AttributeType {
    Unknown,
    MappedAddress,
    Software,
}

impl AttributeType {
    pub fn bytes(&self) -> [u8; 2] {
        match self {
            Self::Unknown => [0x00, 0x00],
            Self::MappedAddress => [0x00, 0x01],
            Self::Software => [0x80, 0x22],
        }
    }

    pub fn from_bytes(bytes: Bytes) -> Self {
        for x in [Self::MappedAddress, Self::Software] {
            if &x.bytes()[..] == bytes {
                return x;
            }
        }
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub attrib_type: AttributeType,
    pub value: Bytes,
}

impl Attribute {
    pub fn bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&self.attrib_type.bytes());
        bytes.put_u16(self.value.len() as u16);
        bytes.put_slice(&self.value);

        // Padding
        while bytes.len() % 4 != 0 {
            log::trace!("Padding");
            bytes.put_u8(0x00);
        }

        bytes.into()
    }

    pub fn from_bytes(bytes: &mut Bytes) -> Self {
        let attrib_type = AttributeType::from_bytes(bytes.copy_to_bytes(2));
        assert_ne!(attrib_type, AttributeType::Unknown);
        let len = bytes.get_u16();
        let value = bytes.copy_to_bytes(len as usize);
        Self { attrib_type, value }
    }
}
