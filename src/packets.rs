mod clientbound;
mod serverbound;

use std::{fmt::Debug, io};
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

struct DecodeError;

#[derive(Error, Debug)]
#[error("error processing incoming packets")]
pub enum PacketParseError {
    #[error("failed to process strings")]
    InvalidUTF8(#[from] std::str::Utf8Error),
    Io(#[from] io::Error),
    #[error("failed to process intent from request")]
    InvalidIntent,
    #[error("packet ID not known by server or unimplemented")]
    UnknownPacketId,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<u8> for PacketParseError {
    fn from(value: u8) -> Self {
        todo!()
    }
}

trait PacketSender {
    fn send();
}

struct ServerResponsePacketSender {}

impl PacketSender for ServerResponsePacketSender {
    fn send() {}
}

fn send_status_response_packet() {}
