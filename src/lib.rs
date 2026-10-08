mod data_types;

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

#[cfg(test)]
mod tests {
    #[test]
    fn packet_handshake_server() {
        // let packets: [u8] = [0x0];
    }
}
