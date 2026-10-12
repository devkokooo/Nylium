pub mod string;
pub mod varint;

use crate::packets::serverbound::PacketParseError;
use std::io::{self, Cursor, Read};

macro_rules! create_cursor_header {
    ($($name:ident => $ty:ty),+ $(,)?) => {
        $(
            #[allow(unused)]
            pub fn $name(cursor: &mut Cursor<&[u8]>) -> io::Result<$ty> {
                let mut buf = [0u8; std::mem::size_of::<$ty>()];
                cursor.read_exact(&mut buf)?;

                Ok(<$ty>::from_be_bytes(buf))
            }
        )+
    };
}

create_cursor_header! {
    read_u8_byte => u8,
    read_u16_short_be => u16,
    read_u32_be => u32,
    read_u64_be => u64,
    read_u128_be => u128,

    read_i8_byte => i8,
    read_i16_short_be => i16,
    read_i32_int_be => i32,
    read_i64_long_be => i64,
    read_i128_long_long_be => i128,

    read_f32_float_be => f32,
    read_f64_double_be => f64,

    read_usize_be => usize,
    read_isize_be => isize,
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

impl TryFrom<i32> for ClientIntent {
    type Error = PacketParseError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ClientIntent::Status),
            2 => Ok(ClientIntent::Login),
            3 => Ok(ClientIntent::Transfer),
            _ => Err(PacketParseError::InvalidIntent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
