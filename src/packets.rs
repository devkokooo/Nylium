pub mod clientbound;
pub mod serverbound;

use std::{
    fmt::Debug,
    io::{self, Cursor, Write},
};

use thiserror::Error;

pub const PROTOCOL_VERSION: i32 = 777;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProtocolState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
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

trait PacketSender {
    fn send();
}

struct ServerResponsePacketSender {}

impl PacketSender for ServerResponsePacketSender {
    fn send() {}
}

fn send_status_response_packet() {}

pub trait PacketEncode {
    fn encode(self, write: impl Write) -> io::Result<usize>;
}

pub trait PacketDecode<'input>: Sized {
    fn decode(cursor: &mut Cursor<&'input [u8]>) -> Result<Self, PacketDecodeError>;
}

#[derive(Error, Debug)]
#[error("error decoding incoming packets")]
pub enum PacketDecodeError {
    Utf8(#[from] std::str::Utf8Error),
    #[error("failed to process strings")]
    Io(#[from] io::Error),
    #[error("failed to process intent from request")]
    InvalidIntent,
    #[error("packet ID not known by server or unimplemented")]
    UnknownPacketId,
}

impl From<u8> for PacketDecodeError {
    fn from(value: u8) -> Self {
        todo!()
    }
}
