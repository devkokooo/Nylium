use std::io::{self, BufRead, Cursor, Seek, SeekFrom, Write};

use thiserror::Error;

/// The 7 least significant bits are used to encode the value and
/// the most significant bit indicates whether there's another byte
/// after it for the next part of the number.
///
/// The least significant group is written first, followed by each
/// of the more significant groups.
///
/// `VarInts` are little endian (however, groups are 7 bits, not 8).
///
/// `VarInts` are never longer than 5 bytes.
///
/// # Examples
/// 10001111 10000111 10000011 00000001
/// |vvvvvvv |vvvvvvv |vvvvvvv |vvvvvvv
/// LE 4     LE 3     LE 2     LE 1
/// <------- <------- <------- <-------
///
/// `VarInt` would be written as:
/// 00000001 10000011 10000111 10001111
/// BE 1     BE 2     BE 3     BE 4
/// -------> -------> -------> ------->
///
/// But we need to drop the first bits and then concatenate.
/// `0000001 0000011 0000111 0001111`
/// `0000 0010_0000 1100_0011 1000_1111`
/// Then interpret concatenated value as u32
///
/// Sample `VarInts`
/// 01111111 -> 127
/// LE `10000000 00000001` -> 128
/// BE `000000___10000000` -> u64
///
/// 10010110 00000001        // Original inputs.
///  0010110  0000001        // Drop continuation bits.
///  0000001  0010110        // Convert to big-endian.
///    00000010010110        // Concatenate.
/// 128 + 16 + 4 + 2 = 150   // Interpret as an unsigned 32-bit integer
///
/// # References
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong>
/// <https://protobuf.dev/programming-guides/encoding/#varints>
pub type VarInt = i32;

#[derive(Error, Debug)]
pub enum VarIntWriteError {
    #[error("write buffer to small")]
    BufferTooSmall,
}

const MAX_VARINT_LENGTH: usize = 5;

/// Reads a sequence of bytes and returns a signed 32-bit VarInt
/// The cursor parameter
pub fn read_var_int(bytes: &mut Cursor<&[u8]>) -> VarInt {
    let mut val: i32 = 0;

    for i in 0..MAX_VARINT_LENGTH {
        let remaining = bytes.fill_buf().unwrap();

        if let Some(byte) = remaining.first() {
            // save bytes in BE form
            val |= (i32::from(*byte) & 0b0111_1111) << (7 * i);

            // Mask first bit to check for next byte
            // if 0 we reached the end of this payload
            if byte & 0b1000_0000 == 0 {
                bytes.consume(1);
                break;
            }
        }
        bytes.consume(1);
    }
    val
}

pub fn write_var_int<W: Write>(val: i32, out: &mut W) -> io::Result<usize> {
    // u32 makes right shifts fill with zeroes
    let mut val = val.cast_unsigned();
    let mut bytes_written: usize = 0;

    for _ in 0..5 {
        let mut data = (val & 0b0111_1111) as u8;
        val >>= 7;

        // If there is data, toggle first bit
        // this helps parser indicate next byte in LE exists
        if val != 0 {
            data |= 0b1000_0000;
        }

        out.write_all(&[data])?;
        bytes_written += 1;

        if val == 0 {
            break;
        }
    }
    Ok(bytes_written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn varint_read() {
        let cases: [(&[u8], i32); _] = [
            (&[0b0000_0000], 0),
            (&[0b0000_0001], 1),
            (&[0b0000_0010], 2),
            (&[0b0111_1111], 127),
            (&[0b1000_0000, 0b0000_0001], 128),
            (&[0b1001_0110, 0b0000_0001], 150),
            (&[0b1111_1111, 0b0000_0001], 255),
            (&[0b1101_1101, 0b1100_0111, 0b0000_0001], 25565),
            (&[0b1111_1111, 0b1111_1111, 0b0111_1111], 2_097_151),
            (
                &[
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_0111,
                ],
                2_147_483_647,
            ),
            (
                &[
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_1111,
                ],
                -1,
            ),
            (
                &[
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b0000_1000,
                ],
                -2_147_483_648,
            ),
        ];

        for (bytes, expected) in cases {
            let mut cursor = Cursor::new(bytes);
            let actual = read_var_int(&mut cursor);

            assert_eq!(actual, expected);
        }
    }

    fn varint_write_vec(val: i32) -> Vec<u8> {
        let mut out = Vec::new();
        write_var_int(val, &mut out).unwrap();
        out
    }

    #[test]
    fn varint_write() {
        let cases: [(i32, &[u8]); _] = [
            (0, &[0b0000_0000]),
            (1, &[0b0000_0001]),
            (2, &[0b0000_0010]),
            (127, &[0b0111_1111]),
            (128, &[0b1000_0000, 0b0000_0001]),
            (150, &[0b1001_0110, 0b0000_0001]),
            (255, &[0b1111_1111, 0b0000_0001]),
            (25565, &[0b1101_1101, 0b1100_0111, 0b0000_0001]),
            (2_097_151, &[0b1111_1111, 0b1111_1111, 0b0111_1111]),
            (
                2_147_483_647,
                &[
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_0111,
                ],
            ),
            (
                -1,
                &[
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_1111,
                ],
            ),
            (
                -2_147_483_648,
                &[
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b0000_1000,
                ],
            ),
        ];

        for (data, expected) in cases {
            let actual = varint_write_vec(data);

            assert_eq!(actual, expected);
        }
    }
}
