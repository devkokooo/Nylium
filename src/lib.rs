mod data_types;

use crate::data_types::*;

pub const PROTOCOL_VERSION: i32 = 777;

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

trait Packet {
    const OFFICIAL_NAME: &'static str;
}

struct DecodeError;
struct PacketProcessingError;

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

enum PacketServerLogin {}
enum PacketServerConfig {}
enum PacketServerPlay {}

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

fn process_incoming_packets(
    packets: &[u8],
    state: ProtocolState,
) -> Result<impl Packet, PacketProcessingError> {
    match state {
        // TODO: extract handshake packet ID, then parse payload into intermediate data types
        ProtocolState::Handshaking => for byte in packets {},
        ProtocolState::Status => {}
        ProtocolState::Login => {}
        ProtocolState::Configuration => {}
        ProtocolState::Play => {}
    }

    Err(PacketProcessingError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_handshake_server_should_be_processed() -> Result<(), Box<dyn std::err::Error>> {
        let state = ProtocolState::Handshaking;

        let packet_id: u8 = 0x0;
        let protocol_version = VarInt(PROTOCOL_VERSION);
        // TODO: prefix string with VarInt bytes for size
        let server_address = String::from("mc.somewhere.com");
        let server_port = UShort(25565);
        let intent = VarInt(0x01);

        let mut packets: Vec<u8> = vec![packet_id];
        packets.extend(protocol_version.to_bytes());
        packets.extend(server_address.clone().into_bytes());
        packets.extend(server_port.0.to_be_bytes());
        packets.extend(intent.to_bytes());

        let actual = process_incoming_packets(&packets, state)?;
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
