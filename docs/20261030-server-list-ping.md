# Server List Ping

https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping

## Current 1.7+

`[Handshake] [C --> S] Client intention`
```json
{
  "host_name": "127.0.0.1",
  "intention": 1,
  "port": 25565,
  "protocol_version": 777,
}
```
**Raw Bytes**
```toml
0x10 # packet size = 16 bytes (as VarInt)
# server state = handshake
0x00 # packet id = 0
0x89 0x06 # protocol version = 777
0x09 0x31 0x32 0x37 0x2E # 0x09 = TAB
0x30 0x2E 0x30 0x2E 0x31 # host name = 127.0.0.1
0x63 0xDD # port = 25565
0x01 # intent = 1 = Status
```

---

`[Status] [C --> S] Status Request`
```json
{}
```

---

`[Status] [S --> C] Status Response`
```json
{
  "status": "{
    \"description\":\"A Minecraft Server\",
    \"players\":{
      \"max\":20,
      \"online\":1,
      \"sample\":[{
        \"id\":\"79db49e8-0abe-3dd6-9e2b-24ca31399f00\",\"name\":\"DevKoko\"
      }]
    },
    \"version\":{
      \"name\":\"26.3\",
      \"protocol\":777
    }
  }"
}
```

`[Status] [C --> S] Ping Request (Status)`
```json
{
  "time": 1654514
}
```

---

`[Status] [S --> C] Pong Response (Status)`
```json
{
  "time": 1654514
}
```

# Packet Format

https://minecraft.wiki/w/Java_Edition_protocol/Packets#Packet_format

Packets cannot be larger than the maximum bytes represented by a 3-byte VarInt. The field length must not be longer than 3 bytes, even if the encoded value is within the limit.

Long encodings at 3 bytes or below are still allowed.

For compressed packets, this applies to the Packet Length field, i.e. the compressed length.

## Without compression

| Field name | Field type |
|------------|------------|
| Length     | VarInt     |
| Packet ID  | VarInt     |
| Data       | Byte Array |
