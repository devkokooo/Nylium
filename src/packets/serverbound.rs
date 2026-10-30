use std::io::Cursor;

use crate::{
    data_types::{
        ClientIntent, UShort,
        varint::{VarInt, decode_varint_value},
    },
    packets::{PacketDecode, PacketDecodeError, ProtocolState, send_status_response_packet},
};

#[derive(Debug, PartialEq)]
pub enum IncomingPacket {
    Handshake(ServerHandshakePacket),
    Status(ServerStatusPacket),
    Login(ServerLoginPacket),
    Config(ServerConfigPacket),
    Play(ServerPlayPacket),
}

#[derive(Debug, PartialEq)]
pub struct ServerHandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: UShort,
    pub intent: ClientIntent,
}

#[derive(Debug, PartialEq)]
pub enum ServerStatusPacket {
    /// The status can only be requested once, immediately after the
    /// handshake, before any ping. The server won't respond otherwise.
    ///
    /// `packet_id`: 0x0
    /// TODO: idempotency on network side? server doesn't respond unless packet is received, before any ping
    StatusRequest,
    PingRequest,
}

#[repr(u8)]
pub enum ServerStatusPacketId {
    StatusRequest = 0x00,
    PingRequest = 0x01,
}

impl TryFrom<u8> for ServerStatusPacketId {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::StatusRequest),
            0x01 => Ok(Self::PingRequest),
            other => Err(other),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerLoginPacket {
    LoginStart,
    EncryptionResponse,
    LoginPluginResponse,
    LoginAcknowledged,
    CookieResponse,
}

#[repr(u8)]
pub enum ServerLoginPacketId {
    LoginStart = 0x00,
    EncryptionResponse = 0x01,
    LoginPluginResponse = 0x02,
    LoginAcknowledged = 0x03,
    CookieResponse = 0x04,
}

#[derive(Debug, PartialEq)]
pub enum ServerConfigPacket {
    ClientInfo,
    CookieResponse,
    PluginMessageServerbound,
    AcknowledgeFinishConfig,
    KeepAliveServerbound,
    Pong,
    ResourcePackResponse,
    KnownPacksServerbound,
    CustomClickAction,
    AcceptCodeOfConduct,
}

#[repr(u8)]
pub enum ServerConfigPacketId {
    ClientInfo = 0x00,
    CookieResponse = 0x01,
    PluginMessageServerbound = 0x02,
    AcknowledgeFinishConfig = 0x03,
    KeepAliveServerbound = 0x04,
    Pong = 0x05,
    ResourcePackResponse = 0x06,
    KnownPacksServerbound = 0x07,
    CustomClickAction = 0x08,
    AcceptCodeOfConduct = 0x09,
}

#[derive(Debug, PartialEq)]
pub enum ServerPlayPacket {}

#[repr(u8)]
pub enum ServerPlayPacketId {
    ConfirmTeleportation = 0x00,
    Attack = 0x01,
    QueryBlockEntityTag = 0x02,
    BundleItemSelected = 0x03,
    ChangeDifficulty = 0x04,
    ChangeGameMode = 0x05,
    AcknowledgeMessage = 0x06,
    ChatCommand = 0x07,
    SignedChatCommand = 0x08,
    ChatMessage = 0x09,
    PlayerSession = 0x0A,
    ChunkBatchReceived = 0x0B,
    ClientStatus = 0x0C,
    ClientTickEnd = 0x0D,
    ClientInfo = 0x0E,
    CommandSuggestionsRequest = 0x0F,
    AcknowledgeConfig = 0x10,
    ClickContainerButton = 0x11,
    ClickContainer = 0x12,
    CloseContainer = 0x13,
    ChangeContainerSlotState = 0x14,
    CookieResponse = 0x15,
    PluginMessageServerbound = 0x16,
    DebugSubscriptionRequest = 0x17,
    EditBook = 0x18,
    QueryEntityTag = 0x19,
    Interact = 0x1A,
    JigsawGenerate = 0x1B,
    KeepAliveServerbound = 0x1C,
    LockDifficulty = 0x1D,
    SetPlayerPosition = 0x1E,
    SetPlayerPositionAndRotation = 0x1F,
    SetPlayerRotation = 0x20,
    SetPlayerMovementFlags = 0x21,
    MoveVehicleServerbound = 0x22,
    PaddleBoat = 0x23,
    PickItemFromBlock = 0x24,
    PickItemFromEntity = 0x25,
    PingRequest = 0x26,
    PlaceRecipe = 0x27,
    PlayerAbilitiesServerbound = 0x28,
    PlayerAction = 0x29,
    PlayerCommand = 0x2A,
    PlayerInput = 0x2B,
    PlayerLoaded = 0x2C,
    Pong = 0x2D,
    Punch = 0x2E,
    ChangeRecipeBookSettings = 0x2F,
    SetSeenRecipe = 0x30,
    RenameItem = 0x31,
    ResourcePackResponse = 0x32,
    SeenAdvancements = 0x33,
    SelectTrade = 0x34,
    SetBeaconEffect = 0x35,
    SetHeldItemServerbound = 0x36,
    ProgramCommandBlock = 0x37,
    ProgramCommandBlockMinecart = 0x38,
    SetCreativeModeSlot = 0x39,
    SetGameRules = 0x3A,
    ProgramJigsawBlock = 0x3B,
    ProgramStructureBlock = 0x3C,
    SetTestBlock = 0x3D,
    UpdateSign = 0x3E,
    SpectatorAction = 0x3F,
    TeleportToEntity = 0x40,
    TestInstanceBlockAction = 0x41,
    UseItemOn = 0x42,
    UseItem = 0x43,
    CustomClockAction = 0x44,
}

#[derive(Debug, PartialEq)]
pub struct ConnectionState {
    pub(crate) server_state: ProtocolState,
    pub(crate) client_intent: Option<ClientIntent>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            server_state: ProtocolState::Handshaking,
            client_intent: None,
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn parse_incoming_packet(
    raw_bytes: &[u8],
    connection_state: &mut ConnectionState,
) -> Result<IncomingPacket, PacketDecodeError> {
    let ConnectionState {
        server_state,
        client_intent,
    } = connection_state;

    let packet_id = raw_bytes[0];
    let packet_payload = &raw_bytes[1..];
    let mut cursor = Cursor::new(packet_payload);

    match server_state {
        ProtocolState::Handshaking => match packet_id {
            0x00 => {
                let data = parse_handshake_packet_payload(&mut cursor, connection_state)?;
                Ok(IncomingPacket::Handshake(data))
            }
            _ => Err(PacketDecodeError::UnknownPacketId),
        },
        ProtocolState::Status => {
            let packet_id = ServerStatusPacketId::try_from(packet_id)?;

            match packet_id {
                ServerStatusPacketId::StatusRequest => {
                    send_status_response_packet();
                    Ok(IncomingPacket::Status(ServerStatusPacket::StatusRequest))
                }
                ServerStatusPacketId::PingRequest => todo!(),
            }
        }
        ProtocolState::Login => todo!(),
        ProtocolState::Configuration => todo!(),
        ProtocolState::Play => todo!(),
    }
}

/// Converts raw bytes from packet into intermediate data types
/// for server use.
///
/// This is done AFTER packet ID has been parsed on SERVERBOUND
/// direction.
fn parse_handshake_packet_payload(
    cursor: &mut Cursor<&[u8]>,
    connection_state: &mut ConnectionState,
) -> Result<ServerHandshakePacket, PacketDecodeError> {
    let VarInt(protocol_version) = decode_varint_value(cursor);
    let server_address = <&str>::decode(cursor)?;
    let server_port = u16::decode(cursor)?;
    let intent = ClientIntent::decode(cursor)?;

    connection_state.client_intent = Some(intent);
    connection_state.server_state = ProtocolState::Status;

    Ok(ServerHandshakePacket {
        protocol_version,
        server_address: String::from(server_address),
        server_port,
        intent,
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        data_types::varint::VarInt,
        packets::{PROTOCOL_VERSION, PacketEncode},
    };

    use super::*;

    // Status ping sequence - vertical slice
    // 1. C -> S: Handshake with Next State set to 1
    // 2. Client and Server set protocol state to Status
    // 3. ...
    //
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What_does_the_normal_status_ping_sequence_look_like?
    #[test]
    fn packet_server_handshake_payload_should_be_processed()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_address = String::from("mc.nyliummc.org");
        let server_port: u16 = 25565;
        let intent = 0x01;

        let mut packets: Vec<u8> = Vec::new();
        VarInt(PROTOCOL_VERSION).encode(&mut packets)?;

        // Minecraft strings need prefixed VarInt for length
        server_address.encode(&mut packets)?;

        server_port.encode(&mut packets)?;
        VarInt(intent).encode(&mut packets)?;

        let mut actual_conn = ConnectionState {
            server_state: ProtocolState::Handshaking,
            client_intent: None,
        };

        let mut cursor = Cursor::new(packets.as_slice());

        let actual = parse_handshake_packet_payload(&mut cursor, &mut actual_conn).unwrap();

        let expected = ServerHandshakePacket {
            protocol_version: 777,
            server_address,
            server_port: 25565,
            intent: ClientIntent::Status,
        };
        let expected_conn = ConnectionState {
            server_state: ProtocolState::Status,
            client_intent: Some(ClientIntent::Status),
        };

        assert_eq!(actual, expected);
        assert_eq!(actual_conn, expected_conn);
        Ok(())
    }

    // We're only handling packet processing individually,
    // not the sequencing yet

    // Status ping sequence - vertical slice
    // 2. ...
    // 3. C -> S: Status Request
    // 4. S -> C: Status Response
    //
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What_does_the_normal_status_ping_sequence_look_like?
    #[test]
    fn packet_server_status_should_be_processed() -> Result<(), Box<dyn std::error::Error>> {
        let default_conn = ConnectionState::default();

        // CASE 0: in handshaking state, should not accept STATUS request

        let mut connection = ConnectionState {
            server_state: ProtocolState::Status,
            client_intent: Some(ClientIntent::Status),
        };

        let packets: Vec<u8> = vec![0x0];

        let actual = parse_incoming_packet(&packets, &mut connection)?;

        assert_eq!(
            actual,
            IncomingPacket::Status(ServerStatusPacket::StatusRequest)
        );

        // CASE 1: not yet requested but has STATUS request -> accept, send STATUS response

        // CASE 2: has been requested, but same STATUS request -> ignore, don't send STATUS response

        // CASE 3: has been requested, then incoming PING request -> accept, send PONG response

        // CASE 4: has not been requested, but incoming PING request -> don't respond

        // CASE 5: has done handshake but incorrect packet ID -> don't respond
        Ok(())
    }
}
