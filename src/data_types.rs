pub mod varint;

#[derive(Debug)]
pub struct Byte(pub i8); // 1 byte

#[derive(Debug)]
pub struct UByte(pub u8); // 1 byte

#[derive(Debug)]
pub struct Short(pub i16); // 2 bytes

#[derive(Debug, PartialEq)]
pub struct UShort(pub u16); // 2 bytes

#[derive(Debug)]
pub struct Int(pub i32); // 4 bytes

#[derive(Debug)]
pub struct Long(pub i64); // 8 bytes

#[derive(Debug)]
pub struct Float(pub f32); // 4 bytes

#[derive(Debug)]
pub struct Double(pub f64); // 8 bytes

/// UTF-8 string prefixed with its size in bytes as a VarInt.
/// Maximum length of `n` characters, which varies by context.
///
/// The encoding used on the wire is regular UTF-8, not
/// Java's "slight modification".
///
/// The length of the string for purposes of the length limit is
/// its number of UTF-16 code units: scalar values > U+FFFF are
/// counted as two.
///
/// Up to `n * 3` bytes can be used to encode a UTF-8 string
/// comprising `n` code units when converted to UTF-16, and both
/// of those limits are checked.
///
/// Maximum `n` value is 32767. The +3 is due to the max size
/// of a valid length VarInt.
///
/// # Why +3 is max size of VarInt?
/// `+3` is the largest number of bytes needed to encode the
/// length prefix, not extra string data. The maximum allowed
/// string length is `n * 4`, with `n <= 32767`, so the largest
/// byte length is `32767 * 4 = 131068`, which fits within
/// 3 VarInt bytes because they carry 21 bits (up to 2,097,151)
// TODO: set the correct data type for String & prefix with VarInt size
#[derive(Debug)]
pub struct ProtoString(String); // 3 VarInt bytes prefix + 4 * n characters

impl ProtoString {
    fn parse(bytes: &[u8]) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct VarLong(pub i64);

#[derive(Debug)]
struct DataTypeError;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
}
