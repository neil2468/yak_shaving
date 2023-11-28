use crate::attribure::Attribute;
use bytes::{BufMut, Bytes, BytesMut};

type MessageMethod = u16;
const BINDING: MessageMethod = 0x0001;

#[derive(PartialEq, Debug)]
pub enum MessageClass {
    Request,
    ResponseSuccess,
    ResponseFailure,
    Indication,
}

#[derive(Debug)]
pub struct MessageType {
    bytes: Bytes,
}


impl MessageType {

    pub fn new(method: MessageMethod, class: MessageClass) -> Self {
        
    }


    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub fn from_bytes(bytes: Bytes) -> Self {
        Self {
            bytes
        }
    }

    pub fn method() -> MessageMethod {
        todo!()
    }

    pub fn class() -> MessageClass {
        todo!()
    }
}






#[derive(Debug)]
pub struct Message {
    pub method: MessageMethod,
    pub class: MessageClass,
    pub id: [u8; 12],
    pub attributes: Vec<Attribute>,
}

impl Message {
    pub fn bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();

        // Attributes
        let mut attribs = BytesMut::new();
        for a in &self.attributes {
            attribs.put(a.bytes());
        }

        // Message Type
        // Assert class is Request.
        // Then we can ignore the class, as Request is zeros.
        assert_eq!(self.class, MessageClass::Request);
        bytes.put_u16(self.method.bytes());

        // Message Length
        // TODO
        bytes.put_u16(attribs.len() as u16);

        // Magic Cookie
        bytes.put_u32(0x2112A442);

        // Transaction ID
        bytes.put_slice(&self.id);

        // Attributes
        bytes.put(attribs);

        bytes.into()
    }

    pub fn from_bytes(bytes: &mut Bytes) -> Self {
        let method = MessageMethod::from_bytes

    }
}
