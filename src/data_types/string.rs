use std::io::{self, Cursor, Write};

use crate::{
    data_types::varint::{VarInt, decode_varint_value},
    packets::{PacketDecode, PacketDecodeError, PacketEncode},
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
impl PacketEncode for &str {
    fn encode(self, mut write: impl Write) -> io::Result<usize> {
        let strlen = i32::try_from(self.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "string too long"))?;

        let mut bytes_written = VarInt(strlen).encode(&mut write)?;
        write.write_all(self.as_bytes())?;
        bytes_written += self.len();
        Ok(bytes_written)
    }
}

impl<'a> PacketDecode<'a> for &'a str {
    fn decode(cursor: &mut Cursor<&'a [u8]>) -> Result<Self, PacketDecodeError> {
        use std::io::BufRead;

        let VarInt(strlen_in_bytes) = decode_varint_value(cursor);
        let strlen_in_bytes = strlen_in_bytes as usize;

        let start = cursor.position() as usize;
        let end = start + strlen_in_bytes;

        let inner_ref: &'a [u8] = cursor.get_ref();
        let str = std::str::from_utf8(&inner_ref[start..end])?;

        cursor.consume(strlen_in_bytes);
        Ok(str)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let cases: [&str; _] = [
            "",
            "a",
            "mc.nyliummc.org",
            "héllo wörld",
            "emoji: 🐨✨🌱",
            "初めまして",
        ];

        for value in cases {
            let mut buf = Vec::new();
            let bytes_written = value.encode(&mut buf).unwrap();

            assert_eq!(bytes_written, buf.len());

            let mut cursor = Cursor::new(&buf[..]);
            let actual: &str = PacketDecode::decode(&mut cursor).unwrap();

            assert_eq!(actual, value);
        }
    }

    #[test]
    fn decode_reads_varint_prefixed_utf8() {
        // "abc" encoded as length-prefixed string: 3, 'a', 'b', 'c'
        let bytes: &[u8] = &[0b0000_0011, b'a', b'b', b'c'];
        let mut cursor = Cursor::new(bytes);

        let actual: &str = PacketDecode::decode(&mut cursor).unwrap();

        assert_eq!(actual, "abc");
    }

    #[test]
    fn encode_prefixes_length_as_varint() {
        let mut buf = Vec::new();
        "abc".encode(&mut buf).unwrap();

        assert_eq!(buf, &[0b0000_0011, b'a', b'b', b'c']);

        let mut buf = Vec::new();
        "nyliummc.org".encode(&mut buf).unwrap();

        assert_eq!(
            buf,
            &[
                0b0000_1100,
                b'n',
                b'y',
                b'l',
                b'i',
                b'u',
                b'm',
                b'm',
                b'c',
                b'.',
                b'o',
                b'r',
                b'g'
            ]
        );
    }

    #[test]
    fn decode_advances_cursor_past_string() {
        let bytes: &[u8] = &[0b0000_0011, b'a', b'b', b'c', 0b0000_0001];
        let mut cursor = Cursor::new(bytes);

        let actual: &str = PacketDecode::decode(&mut cursor).unwrap();

        assert_eq!(actual, "abc");
        assert_eq!(cursor.position(), 4);
    }
}
