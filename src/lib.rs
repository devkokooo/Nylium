use std::error::Error;

enum ProtocolState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtocolDirection {
    Serverbound, // client -> server
    Clientbound, // server -> client
}

/// Figure out the shape of the packets coming in
/// i.e. length of bytes, frame boundaries
fn process_packets_frames(packets: &[u8]) {
    // Iterate byte-by-byte

    // Figure out packet frame boundaries (start and end)

    // Decode packet ID from frame, rest is payload
}

struct Packet {}

struct DecodeError;

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

fn decode_packet(
    state: ProtocolState,
    direction: ProtocolDirection,
    packet_id: i32,
    body: &[u8],
) -> Result<Packet, DecodeError> {
    todo!()
}

/// The 7 least significant bits are used to encode the value and
/// the most significant bit indicates whether there's another byte
/// after it for the next part of the number.
///
/// The least significant group is written first, followed by each
/// of the more significant groups.
///
/// VarInts are little endian (however, groups are 7 bits, not 8).
///
/// VarInts are never longer than 5 bytes.
///
/// # Examples
/// 10001111 10000111 10000011 00000001
/// |vvvvvvv |vvvvvvv |vvvvvvv |vvvvvvv
/// LE 4     LE 3     LE 2     LE 1
/// <------- <------- <------- <-------
///
/// VarInt would be written as:
/// 00000001 10000011 10000111 10001111
///
/// Sample VarInts
/// 01111111 -> 127
/// LE 10000000 00000001 -> 128
/// BE 000000___10000000 -> u64
///
/// 10010110 00000001        // Original inputs.
///  0010110  0000001        // Drop continuation bits.
///  0000001  0010110        // Convert to big-endian.
///    00000010010110        // Concatenate.
/// 128 + 16 + 4 + 2 = 150   // Interpret as an unsigned 64-bit integer
///
/// # References
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
/// https://protobuf.dev/programming-guides/encoding/#varints
struct VarInt(i32);

struct DataTypeError;

impl VarInt {
    fn read_var_int(bytes: &[u8]) -> i32 {
        const MAX_LEN: usize = 5;

        let mut val: i32 = 0;

        for (c, byte) in bytes[0..bytes.len().min(MAX_LEN)]
            .iter()
            .copied()
            .enumerate()
        {
            val |= (byte as i32 & 0x7F) << (7 * c);

            if byte & 0x80 == 0 {
                break;
            }
        }
        val
    }

    fn write_var_int(val: u32) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

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
            (&[0b1111_1111, 0b1111_1111, 0b0111_1111], 2097151),
            (
                &[
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_0111,
                ],
                2147483647,
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
                -2147483648,
            ),
        ];

        for (bytes, expected) in cases {
            let actual = VarInt::read_var_int(bytes);

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn packet_handshake_server() {
        // let packets: [u8] = [0x0];
    }
}
