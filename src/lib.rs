mod data_types;

use std::{
    fmt::Debug,
    io::{self, Cursor},
};

use thiserror::Error;

use crate::data_types::{
    UShort,
    varint::{VarInt, read_var_int},
};

pub const PROTOCOL_VERSION: i32 = 777;

enum ServerState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PacketDirection {
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

/// Packet Format
///
/// Packets cannot be larger than 2^21 - 1 or 2,097,151 bytes (the
/// maximum that can be sent in a 3-byte VarInt). The length field
/// must not be longer than 3 bytes, even if the encoded value is
/// within the limit.
///
/// Unnecessarily long encodings at 3 bytes or below are still
/// allowed. For compressed packets, this applies to the Packet
/// Length field, i.e. the compressed length.
///
/// ## Without compression
/// Length    = VarInt     | Length of Packet ID + Data
/// Packet ID = VarInt     | `protocol_id` from server's packet report
/// Data      = Byte Array | Depends on connection state and packet ID
///
/// ## With compression
/// TODO for later
///
/// # References
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Packet_Format
trait Packet {
    const OFFICIAL_NAME: &'static str;
}

struct DecodeError;

#[derive(Error, Debug)]
#[error("error processing incoming packets")]
struct PacketProcessingError;

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

enum PacketServerLogin {}
enum PacketServerConfig {}
enum PacketServerPlay {}

#[derive(Debug, PartialEq)]
struct PacketServerHandshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: UShort,
    intent: VarInt,
}

impl Packet for PacketServerHandshake {
    const OFFICIAL_NAME: &'static str = "intention";
}

fn official_name<P: Packet>() -> &'static str {
    P::OFFICIAL_NAME
}

type PacketID = i16;

// TODO: extract handshake packet ID, then parse payload into intermediate data types
fn process_incoming_packets(
    packets: &[u8],
    state: ServerState,
) -> Result<impl Packet + Debug, PacketProcessingError> {
    match state {
        ServerState::Handshaking => {
            let packet = process_handshake_packet(&packets);
        }
        ServerState::Status => {}
        ServerState::Login => {}
        ServerState::Configuration => {}
        ServerState::Play => {}
    }

    Ok(todo!() as PacketServerHandshake)
}

fn process_handshake_packet(
    packets: &[u8],
) -> Result<PacketServerHandshake, PacketProcessingError> {
    let mut cursor = Cursor::new(packets);

    println!("cursor pos: {}", cursor.position());
    let protocol_version = read_var_int(&mut cursor);
    println!("cursor pos: {}", cursor.position());
    let string_length = read_var_int(&mut cursor);
    println!("cursor pos: {}", cursor.position());

    let num_bytes_for_string = string_length * 4;
    println!("total bytes for string: {num_bytes_for_string}");

    Ok(PacketServerHandshake {
        protocol_version,
        server_address: String::from("nothing yet"),
        server_port: UShort(0),
        intent: 0,
    })
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, io::Write};

    use crate::data_types::varint::write_var_int;

    use super::*;

    // Status ping sequence - vertical slice
    // 1. C -> S: Handshake with Next State set to 1
    // 2. Client and Server set protocol state to Status
    // 3. ...
    //
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What_does_the_normal_status_ping_sequence_look_like?

    #[test]
    fn packet_handshake_server_should_be_processed() -> Result<(), Box<dyn std::error::Error>> {
        let protocol_version = VarInt::from(PROTOCOL_VERSION);
        let server_address = String::from("mc.somewhere.com");
        let server_port = UShort(25565);
        let intent = VarInt::from(0x01);

        let mut packets: Vec<u8> = Vec::new();
        write_var_int(PROTOCOL_VERSION, &mut packets)?;

        // Minecraft strings need prefixed VarInt for length
        write_var_int(i32::try_from(server_address.len())?, &mut packets)?;
        packets.write_all(server_address.as_bytes())?;

        packets.write_all(&server_port.0.to_be_bytes())?;
        write_var_int(intent, &mut packets)?;

        let actual = process_handshake_packet(&packets).unwrap();
        let expected = PacketServerHandshake {
            protocol_version,
            server_address,
            server_port,
            intent,
        };

        assert_eq!(actual, expected);
        Ok(())
    }
}
