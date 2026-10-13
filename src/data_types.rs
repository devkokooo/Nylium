pub mod string;
pub mod varint;

use crate::{
    data_types::varint::VarInt,
    packets::{PacketDecode, PacketDecodeError, PacketEncode},
};
use std::io::{Cursor, Read, Write};

macro_rules! impl_packet_codec {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl PacketDecode<'_> for $ty {

                #[allow(unused)]
                fn decode(cursor: &mut Cursor<&'_ [u8]>) -> Result<Self, PacketDecodeError> {
                    use std::io::BufRead;

                    let mut buf = [0u8; std::mem::size_of::<$ty>()];
                    cursor.read_exact(&mut buf)?;

                    Ok(<$ty>::from_be_bytes(buf))
                }
            }

            impl PacketEncode for $ty {
                #[allow(unused)]
                fn encode(self, mut write: impl std::io::Write) -> std::io::Result<usize> {
                    write.write(&self.to_be_bytes())
                }
            }
        )+
    };
}

impl_packet_codec! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64, usize, isize,
}

#[derive(Debug)]
pub struct Byte(pub i8); // 1 byte

#[derive(Debug)]
pub struct UByte(pub u8); // 1 byte

pub type Short = i16; // 2 bytes

pub type UShort = u16; // 2 bytes

#[derive(Debug)]
pub struct Int(pub i32); // 4 bytes

#[derive(Debug)]
pub struct Long(pub i64); // 8 bytes

#[derive(Debug)]
pub struct Float(pub f32); // 4 bytes

#[derive(Debug)]
pub struct Double(pub f64); // 8 bytes

#[derive(Debug)]
pub struct VarLong(pub i64);

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(i32)]
pub enum ClientIntent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl PacketDecode<'_> for ClientIntent {
    fn decode(cursor: &mut Cursor<&'_ [u8]>) -> Result<Self, PacketDecodeError> {
        match VarInt::decode(cursor)? {
            VarInt(1) => Ok(ClientIntent::Status),
            VarInt(2) => Ok(ClientIntent::Login),
            VarInt(3) => Ok(ClientIntent::Transfer),
            _ => Err(PacketDecodeError::InvalidIntent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
