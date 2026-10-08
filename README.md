# Nylium
The worst custom Minecraft server implementation in Rust.

Inspired by [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin).

Focus will be [**Java Edition protocol version 777 (26.3)**](https://minecraft.wiki/w/Java_Edition_protocol/Packets)

## Outline Plan
Early Nylium implementation will be a **codec pipeline** to test fixtures in possible server/client packet boundaries, without TCP.

For example, the architecture could follow the data path like so:
```
byte slice
  ➜ frame decoder
  ➜ packet ID + payload
  ➜ state / direction-specific packet decoder
  ➜ typed packet
  ➜ server logic
```
Each arrow can be tested with byte fixtures, without external connections. Byte slices can be `&[u8]` with a cursor / iterator to track offset and return `Result` for truncated or malformed inputs.

TCP adapter can be added later on to feed it complete frame bytes.

### Pipeline Scope
1. **Primitive reader:** `read_u8`, `read_i16_be`, `read_i32_be`, `read_varint`, and length-limited string reader
2. **Frame reader:** decode the length prefix, then expose exactly that many bytes as one frame. No compression until uncompressed version works.
3. **Packet envelope:** decode the packet ID from the frame, leave the rest as payload
4. **Typed example packets:** represent a few packets for protocol state and direction as Rust enums / structs. ID mapping depends on state and direction, avoid single global packet-ID table.
5. **Fixture tests:** test known bytes and expected values, plus truncated input, invalid lengths, oversized VarInts, and trailing bytes

## What are packets?
The Minecraft server accepts connections from TCP clients and communicates with them using **packets**.

A **packet** is a sequence of bytes sent over TCP connection. 

The meaning of packet depends on packet ID and current state of the connection.

The initial state of each connection is **handshaking**, and state is switched using the packets `Handshake` and `Login Success`.

"Serverbound" packets are **received by server from client**.
"Clientbound" packets are **sent from server to client**.

### What is packet framing?
**Packet framing** is how a data link protocol marks the **start and end of each unit of data** sent over a connection.

The frame includes information such as payload, addresses, length, error-check values.

Framing is distinct from **packetization:**
- Packets are network-layer units
- Frames carry packets across a particular link
