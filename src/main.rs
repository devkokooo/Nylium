use nylium::packets::{
    PacketDecodeError,
    serverbound::{ConnectionState, parse_incoming_packet},
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bind listener to address
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

async fn read_available(stream: &mut TcpStream) -> Result<Vec<u8>, PacketDecodeError> {
    // https://minecraft.wiki/w/Java_Edition_protocol/Packets#Without_compression
    let mut packet_len: usize = 0;
    let mut i = 0;

    let mut len_readbuf = [0u8];

    // Packet with compression
    // Prefix Length = VarInt
    // Packet ID = VarInt
    // Data = Byte Array
    // https://minecraft.wiki/w/Java_Edition_protocol/Packets#Without_compression

    // TODO: refactor VarInt helpers to handle this
    loop {
        if i >= 3 {
            return Err(PacketDecodeError::NetworkIngest);
        }

        match stream.read(&mut len_readbuf).await? {
            1 => {
                packet_len = (packet_len << 7) | (len_readbuf[0] as usize & 0x7F);
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
