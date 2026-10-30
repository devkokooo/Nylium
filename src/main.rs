use nylium::packets::{
    PacketDecodeError,
    serverbound::{ConnectionState, parse_incoming_packet},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bind listener to address
    // TODO: read address and port from server.properties
    let addr = "127.0.0.1:25565";
    let listener = TcpListener::bind(addr).await?;
    println!("Binding to: {addr}");

    let mut connection_state = ConnectionState::default();

    loop {
        // second item contains IP and port of the new connection
        let (socket, _) = listener.accept().await?;
        process_socket(socket, &mut connection_state).await?;
    }
}

async fn process_socket(
    mut socket: TcpStream,
    connection_state: &mut ConnectionState,
) -> Result<(), Box<dyn std::error::Error>> {
    let local = socket.local_addr()?;
    let remote = socket.peer_addr()?;

    // [1] [Handshake] [C --> S] Client Intention
    // [2] [Status] [C --> S] Status Request
    // [3] [Status] [S --> C] Status Response
    // [4] [Status] [C --> S] Ping Request (Status)
    // [5] [Status] [S --> C] Pong Response (Status)
    socket.readable().await?;
    println!("local: {local:#?}, remote: {remote:#?}");

    let packets = read_available(&mut socket).await?;
    println!("{packets:02x?}");

    if let Ok(res) = parse_incoming_packet(&packets, connection_state) {
        println!("{connection_state:#?}");
        println!("{res:#?}");
    }

    Ok(())
}

async fn read_packet<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await?;

    Ok(bytes)
}

async fn read_varint_prefix_len<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<usize> {
    let mut val = 0usize;

    for i in 0..3 {
        let byte = reader.read_u8().await?;

        // Each byte contributes 7 bits
        let shift = i * 7;
        val |= usize::from(byte & 0x7F)
            .checked_shl(shift as u32)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "VarInt overflow")
            })?;

        if byte & 0x80 == 0 {
            return Ok(val);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "VarInt exceeds 3 bytes",
    ))
}

/// Read available packets without compression
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Without_compression>
///
/// Packet without compression has the following shape:
/// Length    = `VarInt` (length of packet + data)
/// Packet ID = `VarInt`
/// Data      = Byte Array (depends on connection state and packet ID)
async fn read_available(stream: &mut TcpStream) -> Result<Vec<u8>, PacketDecodeError> {
    let mut packet_len: usize = 0;
    let mut i = 0;

    let mut prefix_len = [0u8];

    // TODO: refactor VarInt helpers to handle this
    // read the next 1-3 bytes as VarInt to find prefix length
    // THEN read the packet within the frames
    loop {
        // Packet length cannot be larger than 3-byte VarInt
        if i >= 3 {
            return Err(PacketDecodeError::NetworkIngest);
        }

        match stream.read(&mut prefix_len).await? {
            1 => {
                packet_len = (packet_len << 7) | (prefix_len[0] as usize & 0x7F);
                if packet_len & 0x80 == 0 {
                    break;
                }
            }
            0 => return Err(PacketDecodeError::NetworkIngest),
            _ => unreachable!(),
        }

        i += 1;
    }

    let mut packets = vec![0u8; packet_len];
    stream.read_exact(&mut packets).await?;

    Ok(packets)
}

#[ignore = "wip"]
#[tokio::test]
async fn reads_mock_bytes() {
    let mock_bytes = [
        0x10, // packet size = 16 bytes (as VarInt)
        0x00, // packet id = 0
        0x89, 0x06, // protocol version = 777
        0x09, 0x31, 0x32, 0x37, 0x2E, // 0x09 = TAB
        0x30, 0x2E, 0x30, 0x2E, 0x31, // host name = 127.0.0.1
        0x63, 0xDD, // port = 25565
        0x01, // intent = 1 = Status
    ];

    let mut mock = tokio_test::io::Builder::new().read(&mock_bytes).build();

    // let packets = read_available(&mut mock).await.unwrap();
}
