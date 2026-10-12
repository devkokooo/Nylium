use std::io::{self, BufRead, Cursor, Write};

use crate::{
    data_types::varint::{read_var_int, write_var_int},
    packets::clientbound::PacketWriteError,
};

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
pub fn parse_packet_string<'a>(
    cursor: &mut Cursor<&'a [u8]>,
) -> Result<&'a str, std::str::Utf8Error> {
    let strlen_in_bytes = read_var_int(cursor);
    let str = read_string(cursor, strlen_in_bytes as usize)?;

    Ok(str)
}

pub fn read_string<'a>(
    cursor: &mut Cursor<&'a [u8]>,
    length_in_bytes: usize,
) -> Result<&'a str, std::str::Utf8Error> {
    let start = cursor.position() as usize;
    let end = start + length_in_bytes;

    let inner_ref: &'a [u8] = cursor.get_ref();
    let str = std::str::from_utf8(&inner_ref[start..end])?;

    cursor.consume(length_in_bytes);
    Ok(str)
}

pub fn write_string<W: Write>(str: &str, out: &mut W) -> Result<usize, PacketWriteError> {
    let strlen = str.len();
    let bytes = str.as_bytes();

    let _ = write_var_int(i32::try_from(strlen)?, out);
    let _ = out.write_all(bytes);

    Ok(str.len())
}

#[cfg(test)]
mod tests {
    use crate::data_types::varint::write_var_int;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn string_should_be_read() -> Result<(), std::str::Utf8Error> {
        let str = "mc.nyliummc.org".as_bytes();
        let str_len = str.len();

        let mut cursor = Cursor::new(str);

        let actual = read_string(&mut cursor, str_len)?;
        let expected = "mc.nyliummc.org";

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn packet_string_should_be_parsed() -> Result<(), Box<dyn std::error::Error>> {
        let str = "mc.nyliummc.org".as_bytes();
        let str_len = str.len();

        let mut strlen_varint = Vec::new();
        write_var_int(i32::try_from(str_len)?, &mut strlen_varint).unwrap();

        strlen_varint.extend(str);
        let packet = strlen_varint.as_slice();

        let mut cursor = Cursor::new(packet);

        let actual = parse_packet_string(&mut cursor)?;
        let expected = "mc.nyliummc.org";

        assert_eq!(actual, expected);
        Ok(())
    }

    fn string_write_vec(str: &str) -> Vec<u8> {
        let mut out = Vec::new();
        write_string(str, &mut out).unwrap();
        out
    }

    #[test]
    fn string_should_be_written_into_packet() -> Result<(), Box<dyn std::error::Error>> {
        let str = "mc.nyliummc.org"; // 15 char * 4 = 60 bytes

        let actual = string_write_vec(str);

        let mut expected: Vec<u8> = Vec::new();
        let str_bytes = str.as_bytes();
        write_var_int(i32::try_from(str.len())?, &mut expected)?;
        expected.extend(str_bytes);

        assert_eq!(actual, expected);
        Ok(())
    }
}
