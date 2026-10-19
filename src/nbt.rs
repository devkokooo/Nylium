use std::io::{Cursor, Read};

use thiserror::Error;

pub trait Decode<'input>: Sized {
    fn decode(cursor: &mut Cursor<&'input [u8]>) -> Result<Self, NBTParseError>;
}

#[derive(PartialEq)]
pub enum NBTTagType {
    /// Used to mark the end of compound tags.
    /// It is always a single byte 0.
    /// It may also be the type of empty List tags.
    End = 0x00,
    /// 1 byte / 8 bits, signed
    /// Sometimes used for booleans.
    ///
    ///           1 byte      A    1 byte
    /// [0x01] [0x00 0x01] [0x41] [0xFF]
    /// tag ID  name len    name  payload
    Byte = 0x01,
    /// 2 bytes / 16 bits, signed, big endian
    ///
    ///           1 byte      A     2 bytes
    /// [0x02] [0x00 0x01] [0x41] [0xFF 0xFF]
    /// tag ID  name len    name    payload
    Short = 0x02,
    /// 4 bytes / 32 bits, signed, big endian
    ///
    ///           1 byte      A          4 bytes
    /// [0x03] [0x00 0x01] [0x41] [0xFF 0xFF 0xFF 0xFF]
    /// tag ID  name len    name         payload
    Int = 0x03,
    /// 8 bytes / 64 bits, signed, big endian
    ///
    ///           1 byte      A                    8 bytes
    /// [0x04] [0x00 0x01] [0x41] [0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF]
    /// tag ID  name len    name                   payload
    Long = 0x04,
    /// 4 bytes / 32 bits, signed, big endian,
    ///
    ///           1 byte      A          4 bytes
    /// [0x05] [0x00 0x01] [0x41] [0xFF 0xFF 0xFF 0xFF]
    /// tag ID  name len    name         payload
    Float = 0x05,
    /// 8 bytes / 64 bits, signed, big endian
    ///
    ///           1 byte      A                    8 bytes
    /// [0x06] [0x00 0x01] [0x41] [0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF]
    /// tag ID  name len    name                   payload
    Double = 0x06,
    /// 4 bytes / 32 bits, signed, big endian for size, then bytes of length size
    ///
    ///           1 byte      A          4 bytes
    /// [0x07] [0x00 0x01] [0x41] [0x00 0x00 0x00 0x02] [0xFF 0xFF]
    /// tag ID  name len    name        array len         payload
    ByteArray = 0x07,
    /// 2 bytes / 16 bits, unsigned, big endian for size, then bytes of length size as UTF-8
    /// no null termination
    ///
    ///           1 byte      A     6 bytes          6 bytes of UTF-8
    /// [0x08] [0x00 0x01] [0x41] [0x00 0x06] [0xFF 0xFF 0xFF 0xFF 0xFF 0xFF]
    /// tag ID  name len    name   data len              payload
    String = 0x08,
    /// 1 byte / 8 bits for tag ID of the list's contents, then
    /// 4 bytes / 32 bits, big endian for size, then
    /// length size number of items of tag ID
    ///
    ///           1 byte      A
    /// [0x09] [0x00 0x01] [0x41] [0x02]       [0x00 0x00 0x00 0x01] [[Short]]
    /// tag ID  name len    name  list tag ID        list size        payload
    List = 0x09,
    /// Contains any number of tags, delimited by `TAG_End`.
    /// Each tag consisting of 1 byte / 8 bits tag ID, followed by
    /// 2 bytes / 16 bits, unsigned, big endian for size, then
    /// UTF-8 formatted string containing the tag name.
    /// Payload data is last.
    ///
    /// [0x0A] [0x00 0x00] [   ] [0x08, 0x00, 0x06 ...]
    /// tag ID    size     name       payload
    ///
    ///                      r    o    o    t
    /// [0x0A] [0x00 0x04] [0x72 0x6F 0x6F 0x74] [0x01 0x00 0x00 0x00 0x00 ...] [0x00]
    /// tag ID  name size        tag name         payload                        End
    ///
    /// Although there is no hard limit to the number of tags
    /// within a Compound, note that Compound and List tags
    /// maybe not be nested beyond a depth of 512.
    Compound = 0x0A,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of `TAG_Int` payloads.
    ///
    ///           1 byte      A          4 bytes            
    /// [0x0B] [0x00 0x01] [0x41] [0x00 0x00 0x00 0x02] [[0xFF 0xFF 0xFF 0xFF][0xFF 0xFF 0xFF 0xFF]]
    /// tag ID  name len    name        array len        Int payloads
    IntArray = 0x0B,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of `TAG_Long` payloads.
    ///
    ///           1 byte      A          4 bytes            
    /// [0x0C] [0x00 0x01] [0x41] [0x00 0x00 0x00 0x01] [[0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF]]
    /// tag ID  name len    name        array len        Long payloads
    LongArray = 0x0C,
}

impl TryFrom<u8> for NBTTagType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::End),
            0x01 => Ok(Self::Byte),
            0x02 => Ok(Self::Short),
            0x03 => Ok(Self::Int),
            0x04 => Ok(Self::Long),
            0x05 => Ok(Self::Float),
            0x06 => Ok(Self::Double),
            0x07 => Ok(Self::ByteArray),
            0x08 => Ok(Self::String),
            0x09 => Ok(Self::List),
            0x0A => Ok(Self::Compound),
            0x0B => Ok(Self::IntArray),
            0x0C => Ok(Self::LongArray),
            other => Err(other),
        }
    }
}

impl std::fmt::Display for NBTTagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::End => "NBTTag::End",
            Self::Byte => "NBTTag::Byte",
            Self::Short => "NBTTag::Short",
            Self::Int => "NBTTag::Int",
            Self::Long => "NBTTag::Long",
            Self::Float => "NBTTag::Float",
            Self::Double => "NBTTag::Double",
            Self::ByteArray => "NBTTag::ByteArray",
            Self::String => "NBTTag::String",
            Self::List => "NBTTag::List",
            Self::Compound => "NBTTag::Compound",
            Self::IntArray => "NBTTag::IntArray",
            Self::LongArray => "NBTTag::LongArray",
            _ => "Unknown Tag",
        };

        write!(f, "{label}")
    }
}

#[derive(Debug, PartialEq)]
pub enum NBTNode {
    EndMarker,
    Byte {
        name: String,
        value: i8,
    },
    Short {
        name: String,
        value: i16,
    },
    Int {
        name: String,
        value: i32,
    },
    Long {
        name: String,
        value: i64,
    },
    Float {
        name: String,
        value: f32,
    },
    Double {
        name: String,
        value: f64,
    },
    ByteArray {
        name: String,
        values: Vec<u8>,
    },
    String {
        name: String,
        value: String,
    },
    List {
        name: String,
        values: Vec<NBTPayloadType>,
    },
    Compound {
        name: String,
        children: Vec<NBTNode>,
    },
    IntArray {
        name: String,
        values: Vec<i32>,
    },
    LongArray {
        name: String,
        values: Vec<i64>,
    },
}

#[derive(Debug, PartialEq)]
pub enum NBTPayloadType {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
}

#[derive(Debug, Error)]
#[error("NBT parse error")]
pub enum NBTParseError {
    PayloadTooSmall,
    SliceConversion(#[from] std::array::TryFromSliceError),
    TypeConversion(#[from] std::num::TryFromIntError),
    Utf8(#[from] std::str::Utf8Error),
    Io(#[from] std::io::Error),

    #[error("error parsing from iu8: {0}")]
    FromU8(u8),

    #[error("error parsing from i8: {0}")]
    FromI8(i8),
}

impl From<u8> for NBTParseError {
    fn from(value: u8) -> Self {
        Self::FromU8(value)
    }
}

impl From<i8> for NBTParseError {
    fn from(value: i8) -> Self {
        Self::FromI8(value)
    }
}

pub fn parse_nbt_payload(mut cursor: &mut Cursor<&[u8]>) -> Result<NBTNode, NBTParseError> {
    if cursor.get_ref().len() < 3 {
        return Err(NBTParseError::PayloadTooSmall);
    }

    let mut tag = [0u8];
    cursor.read_exact(&mut tag)?;
    let tag = NBTTagType::try_from(tag[0])?;

    // the ONLY tag without name length bytes
    if tag == NBTTagType::End {
        return Ok(NBTNode::EndMarker);
    }

    let name = parse_nbt_name(cursor)?;
    let name_len = name.len();

    println!("{tag}, name={name}, name_len={name_len}");

    match tag {
        NBTTagType::Compound => {
            let mut children: Vec<NBTNode> = Vec::new();

            loop {
                let node = parse_nbt_payload(cursor)?;

                if node == NBTNode::EndMarker {
                    break;
                }
                children.push(node);
            }

            let compound = NBTNode::Compound { name, children };

            return Ok(compound);
        }
        NBTTagType::Byte => {
            let mut payload = [0u8];
            cursor.read_exact(&mut payload)?;
            let payload = i8::try_from(payload[0])?;

            let byte = NBTNode::Byte {
                name,
                value: payload,
            };

            return Ok(byte);
        }
        NBTTagType::Short => {
            let mut payload = [0u8; 2];
            cursor.read_exact(&mut payload)?;
            let payload = i16::from_be_bytes(payload);

            let short = NBTNode::Short {
                name,
                value: payload,
            };

            return Ok(short);
        }
        NBTTagType::Int => {
            let mut payload = [0u8; 4];
            cursor.read_exact(&mut payload)?;
            let payload = i32::from_be_bytes(payload);

            let int = NBTNode::Int {
                name,
                value: payload,
            };

            return Ok(int);
        }
        NBTTagType::Long => {
            let mut payload = [0u8; 8];
            cursor.read_exact(&mut payload)?;
            let payload = i64::from_be_bytes(payload);

            let long = NBTNode::Long {
                name,
                value: payload,
            };

            return Ok(long);
        }
        NBTTagType::Float => {
            let mut payload = [0u8; 4];
            cursor.read_exact(&mut payload)?;
            let payload = f32::from_be_bytes(payload);

            let float = NBTNode::Float {
                name,
                value: payload,
            };

            return Ok(float);
        }
        NBTTagType::Double => {
            let mut payload = [0u8; 8];
            cursor.read_exact(&mut payload)?;
            let payload = f64::from_be_bytes(payload);

            let double = NBTNode::Double {
                name,
                value: payload,
            };

            return Ok(double);
        }
        NBTTagType::ByteArray => {
            let mut arr_len = [0u8; 4];
            cursor.read_exact(&mut arr_len)?;
            let arr_len = i32::from_be_bytes(arr_len);

            let mut bytes = vec![0u8; arr_len as usize];
            cursor.read_exact(&mut bytes)?;

            let byte_arr = NBTNode::ByteArray {
                name,
                values: bytes,
            };

            return Ok(byte_arr);
        }
        NBTTagType::IntArray => {
            let mut arr_len = [0u8; 4];
            cursor.read_exact(&mut arr_len)?;
            let arr_len = i32::from_be_bytes(arr_len);

            let mut ints = vec![0i32; arr_len as usize];
            let mut payload = [0u8; 4];

            for i in 0..arr_len {
                cursor.read_exact(&mut payload)?;
                let payload = i32::from_be_bytes(payload);

                ints[i as usize] = payload;
            }

            let int_arr = NBTNode::IntArray { name, values: ints };

            return Ok(int_arr);
        }
        NBTTagType::LongArray => {
            let mut arr_len = [0u8; 4];
            cursor.read_exact(&mut arr_len)?;
            let arr_len = i32::from_be_bytes(arr_len);

            let mut longs = vec![0i64; arr_len as usize];
            let mut payload = [0u8; 8];

            for i in 0..arr_len {
                cursor.read_exact(&mut payload)?;
                let payload = i64::from_be_bytes(payload);

                longs[i as usize] = payload;
            }

            let long_arr = NBTNode::LongArray {
                name,
                values: longs,
            };

            return Ok(long_arr);
        }
        NBTTagType::String => {
            let payload = parse_nbt_string(cursor)?;
            let string = NBTNode::String {
                name,
                value: payload,
            };

            return Ok(string);
        }
        NBTTagType::List => {
            // list item tag ID
            let mut item_id = [0u8];
            cursor.read_exact(&mut item_id)?;
            let item_id = u8::from_be_bytes(item_id);
            let item_id = NBTTagType::try_from(item_id)?;

            // list size - 4 bytes
            let mut list_size = [0u8; 4];
            cursor.read_exact(&mut list_size)?;
            let list_size = u32::from_be_bytes(list_size) as usize;

            // payloads
            let mut arr: Vec<NBTPayloadType> = Vec::new();

            match item_id {
                NBTTagType::String => {
                    for _ in 0..list_size {
                        let payload = parse_nbt_string(cursor)?;

                        arr.push(NBTPayloadType::String(payload));
                    }
                }
                _ => {}
            }

            let list = NBTNode::List { name, values: arr };

            return Ok(list);
        }
        _ => {}
    }

    Ok(NBTNode::Byte {
        name: "fake".to_owned(),
        value: 0,
    })
}

/// Parses first 2 bytes (u16) for name length
/// Then parses name length bytes for the name
/// Returns empty string if no name
fn parse_nbt_name(cursor: &mut Cursor<&[u8]>) -> Result<String, NBTParseError> {
    let mut name_len = [0u8; 2];
    cursor.read_exact(&mut name_len)?;
    let name_len = u16::from_be_bytes(name_len) as usize;

    if name_len == 0 {
        return Ok(String::new());
    }

    let mut name = vec![0u8; name_len];
    cursor.read_exact(&mut name)?;
    let name = std::str::from_utf8(&name)?;

    Ok(name.to_owned())
}

fn parse_nbt_string(cursor: &mut Cursor<&[u8]>) -> Result<String, NBTParseError> {
    let mut str_len = [0u8; 2];
    cursor.read_exact(&mut str_len)?;
    let str_len = u16::from_be_bytes(str_len) as usize;

    let mut payload = vec![0u8; str_len];
    cursor.read_exact(&mut payload)?;
    let payload = str::from_utf8(&payload)?.to_owned();

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn nbt_compound_parsing() -> Result<(), NBTParseError> {
        let biome_palette: [u8; _] = [
            0x0a, // Compound tag
            0x00, 0x06, // name length = 6
            0x62, 0x69, 0x6f, 0x6d, 0x65, 0x73, // name = biomes
            0x09, // List tag
            0x00, 0x07, // name length = 7
            0x70, 0x61, 0x6c, 0x65, 0x74, 0x74, 0x65, // name = palette
            0x08, // list item tag id = String
            0x00, 0x00, 0x00, 0x01, // list size = 1
            // payloads
            0x00, 0x10, // str length = 16
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, //
            0x74, 0x3a, 0x70, 0x6c, 0x61, 0x69, 0x6e, 0x73, // minecraft:plains
            0x00, // END MARKER
        ];
        let expected_biome = NBTNode::Compound {
            name: "biomes".to_owned(),
            children: Vec::from([NBTNode::List {
                name: "palette".to_owned(),
                values: Vec::from([NBTPayloadType::String("minecraft:plains".to_owned())]),
            }]),
        };

        // Expected should be:
        // root NBTCompound { NBTString Status = "minecraft:full" }
        let multiple_types: [u8; _] = [
            0x0a, // Compound tag
            0x00, 0x00, // name length = 0
            // everything after is payload since no name
            0x08, // String tag
            0x00, 0x06, // name length = 6
            0x53, 0x74, 0x61, 0x74, 0x75, 0x73, // name = Status
            0x00, 0x0e, // payload length = 14
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, //
            0x66, 0x74, 0x3a, 0x66, 0x75, 0x6c, 0x6c, // minecraft:full
            0x03, // Int tag
            0x00, 0x04, // name length = 4
            0x7a, 0x50, 0x6f, 0x73, // name = zPos
            0x00, 0x00, 0x00, 0x00, // value = 0
            0x00, // END MARKER
        ];
        let expected_multiple = NBTNode::Compound {
            name: String::new(),
            children: Vec::from([
                NBTNode::String {
                    name: "Status".to_owned(),
                    value: "minecraft:full".to_owned(),
                },
                NBTNode::Int {
                    name: "zPos".to_owned(),
                    value: 0,
                },
            ]),
        };

        let nested_compounds: [u8; _] = [
            0x0a, // Compound tag
            0x00, 0x0a, // name length = 10
            0x73, 0x74, 0x72, 0x75, 0x63, //
            0x74, 0x75, 0x72, 0x65, 0x73, // name = structures
            // everything after name is the payload
            // this is a nested compound payload
            0x0a, // Compound tag
            0x00, 0x0a, // name length = 10
            0x52, 0x65, 0x66, 0x65, 0x72, //
            0x65, 0x6e, 0x63, 0x65, 0x73, // name = References
            0x00, // End tag, mark end of compound
            0x0a, // Compound tag
            0x00, 0x06, // name length = 6
            0x73, 0x74, 0x61, 0x72, 0x74, 0x73, // name = starts
            0x00, // End tag, mark end of compound
            0x00, // End tag for end of parent compound
        ];
        let expected_nested = NBTNode::Compound {
            name: "structures".to_owned(),
            children: Vec::from([
                NBTNode::Compound {
                    name: "References".to_owned(),
                    children: Vec::new(),
                },
                NBTNode::Compound {
                    name: "starts".to_owned(),
                    children: Vec::new(),
                },
            ]),
        };

        let cases: [(&[u8], NBTNode); _] = [
            (&biome_palette, expected_biome),
            (&multiple_types, expected_multiple),
            (&nested_compounds, expected_nested),
        ];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_byte_parsing() -> Result<(), NBTParseError> {
        // Y = 64
        let byte_data: [u8; _] = [
            0x01, // tag ID
            0x00, 0x01, // name size - u16
            0x59, // name (Y) - UTF-8 bytes
            0x40, // payload (byte, 64) - 1 byte, signed
        ];
        let expected_byte = NBTNode::Byte {
            name: "Y".to_owned(),
            value: 64,
        };

        // Mob = true
        let bool_data: [u8; _] = [
            0x01, // tag ID
            0x00, 0x03, // name size - u16
            0x4D, 0x6F, 0x62, // name (Mob) - UTF-8 bytes
            0x01, // payload (bool, true) - 1 byte, signed
        ];
        let expected_bool = NBTNode::Byte {
            name: "Mob".to_owned(),
            value: 1, // 1 for true, 0 for false
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&byte_data, expected_byte), (&bool_data, expected_bool)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_short_parsing() -> Result<(), NBTParseError> {
        let bytes_zero: [u8; _] = [
            0x02, // Short tag
            0x00, 0x06, // name length = 6
            0x61, 0x70, 0x70, 0x6C, 0x65, 0x73, // name = apples
            // payload = 0
            0x00, 0x00,
        ];
        let expected_zero = NBTNode::Short {
            name: "apples".to_owned(),
            value: 0,
        };

        let bytes_some: [u8; _] = [
            0x02, // Short tag
            0x00, 0x06, // name length = 6
            0x61, 0x70, 0x70, 0x6C, 0x65, 0x73, // name = apples
            // payload = 20
            0x00, 0x14,
        ];
        let expected_some = NBTNode::Short {
            name: "apples".to_owned(),
            value: 20,
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&bytes_zero, expected_zero), (&bytes_some, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_int_parsing() -> Result<(), NBTParseError> {
        let bytes_zero: [u8; _] = [
            0x03, // Int tag
            0x00, 0x04, // name length = 4
            0x7a, 0x50, 0x6f, 0x73, // name = zPos
            0x00, 0x00, 0x00, 0x00, // value = 0
        ];
        let expected_zero = NBTNode::Int {
            name: "zPos".to_owned(),
            value: 0,
        };

        let bytes_some: [u8; _] = [
            0x03, // Int tag
            0x00, 0x04, // name length = 4
            0x79, 0x50, 0x6f, 0x73, // name = yPos
            0xff, 0xff, 0xff, 0xfc, // value = -4
        ];
        let expected_some = NBTNode::Int {
            name: "yPos".to_owned(),
            value: -4,
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&bytes_zero, expected_zero), (&bytes_some, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_long_parsing() -> Result<(), NBTParseError> {
        let bytes_zero: [u8; _] = [
            0x04, // Long tag
            0x00, 0x0a, // name length = 10
            0x4c, 0x61, 0x73, 0x74, 0x55, //
            0x70, 0x64, 0x61, 0x74, 0x65, // name = LastUpdate
            // payload = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_zero = NBTNode::Long {
            name: "LastUpdate".to_owned(),
            value: 0,
        };

        let bytes_some: [u8; _] = [
            0x04, // Long tag,
            0x00, 0x0a, // name length = 10
            0x4c, 0x61, 0x73, 0x74, 0x55, //
            0x70, 0x64, 0x61, 0x74, 0x65, // name = LastUpdate
            // payload = 8589934591
            0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected_some = NBTNode::Long {
            name: "LastUpdate".to_owned(),
            value: 8_589_934_591,
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&bytes_zero, expected_zero), (&bytes_some, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_float_parsing() -> Result<(), NBTParseError> {
        let bytes_zero: [u8; _] = [
            0x05, // Float tag
            0x00, 0x09, // name length = 9
            0x69, 0x6E, 0x74, 0x65, 0x67, //
            0x72, 0x69, 0x74, 0x79, // name = integrity
            // payload = 0
            0x00, 0x00, 0x00, 0x00,
        ];
        let expected_zero = NBTNode::Float {
            name: "integrity".to_owned(),
            value: 0.0,
        };

        let bytes_some: [u8; _] = [
            0x05, // Float tag
            0x00, 0x09, // name length = 9
            0x69, 0x6E, 0x74, 0x65, 0x67, //
            0x72, 0x69, 0x74, 0x79, // name = integrity
            // payload = 6.3108872e-30
            0x0F, 0x00, 0x00, 0x00,
        ];
        let expected_some = NBTNode::Float {
            name: "integrity".to_owned(),
            value: 6.310_887_2e-30,
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&bytes_zero, expected_zero), (&bytes_some, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_double_parsing() -> Result<(), NBTParseError> {
        let bytes_zero: [u8; _] = [
            0x06, // Double tag
            0x00, 0x06, // name length = 6
            0x68, 0x65, 0x69, 0x67, 0x68, 0x74, // name = height
            // payload = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_zero = NBTNode::Double {
            name: "height".to_owned(),
            value: 0.0,
        };

        let bytes_some: [u8; _] = [
            0x06, // Double tag
            0x00, 0x06, // name length = 6
            0x68, 0x65, 0x69, 0x67, 0x68, 0x74, // name = height
            // payload = 999.99
            0x40, 0x8F, 0x3F, 0xEB, 0x85, 0x1E, 0xB8, 0x52,
        ];
        let expected_some = NBTNode::Double {
            name: "height".to_owned(),
            value: 999.99,
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&bytes_zero, expected_zero), (&bytes_some, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_string_parsing() -> Result<(), NBTParseError> {
        let bytes: [u8; _] = [
            0x08, // String tag
            0x00, 0x06, // name length = 6
            0x53, 0x74, 0x61, 0x74, 0x75, 0x73, // name = Status
            0x00, 0x0e, // payload length = 14
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, //
            0x66, 0x74, 0x3a, 0x66, 0x75, 0x6c, 0x6c, // minecraft:full
        ];

        let mut cursor = Cursor::new(&bytes[..]);

        let actual = parse_nbt_payload(&mut cursor)?;
        let expected = NBTNode::String {
            name: "Status".to_owned(),
            value: "minecraft:full".to_owned(),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn nbt_list_parsing() -> Result<(), NBTParseError> {
        let empty_list: [u8; _] = [
            0x09, // List tag
            0x00, 0x0e, // name length = 14
            0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x5f, 0x65, //
            0x6e, 0x74, 0x69, 0x74, 0x69, 0x65, 0x73, // name = block_entities
            0x00, // list tag ID = 0
            0x00, 0x00, 0x00, 0x00, // list size = 0, num of items of tag ID
        ];
        let expected_empty = NBTNode::List {
            name: "block_entities".to_owned(),
            values: Vec::new(),
        };

        let some_list: [u8; _] = [
            0x09, // List tag
            0x00, 0x07, // name length = 7
            0x70, 0x61, 0x6c, 0x65, 0x74, 0x74, 0x65, // name = palette
            0x08, // list item tag ID
            0x00, 0x00, 0x00, 0x01, // num list items = 1
            0x00, 0x10, // payload size = 16
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, //
            0x74, 0x3a, 0x70, 0x6c, 0x61, 0x69, 0x6e, 0x73, // minecraft:plains
        ];
        let expected_some = NBTNode::List {
            name: "palette".to_owned(),
            values: Vec::from([NBTPayloadType::String("minecraft:plains".to_owned())]),
        };

        let cases: [(&[u8], NBTNode); _] =
            [(&empty_list, expected_empty), (&some_list, expected_some)];

        for (data, expected) in cases {
            let mut cursor = Cursor::new(data);

            let actual = parse_nbt_payload(&mut cursor)?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn nbt_byte_array_parsing() -> Result<(), NBTParseError> {
        let bytes_some: [u8; _] = [
            0x07, // Byte array tag
            0x00, 0x0a, // name length = 10
            0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x4c, //
            0x69, 0x67, 0x68, 0x74, // name = BlockLight
            0x00, 0x00, 0x00, 0x08, // payload size = 8
            0xff, 0xff, 0xff, 0xff, // 8 blocks represented, light level 15
            0xff, 0xff, 0xff, 0xfe, // 8 blocks represented, light level 15 to 14
        ];

        let mut cursor = Cursor::new(&bytes_some[..]);

        let actual = parse_nbt_payload(&mut cursor)?;
        let expected = NBTNode::ByteArray {
            name: "BlockLight".to_owned(),
            values: Vec::from([
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1110,
            ]),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn nbt_int_array_parsing() -> Result<(), NBTParseError> {
        let bytes_some: [u8; _] = [
            0x0b, // Int array tag
            0x00, 0x02, // name length = 2
            0x42, 0x42, // name = BB (for bounding box)
            0x00, 0x00, 0x00, 0x06, // payload size = 6
            0x00, 0x00, 0x00, 0x00, // min X
            0x00, 0x00, 0x00, 0x00, // min Y
            0x00, 0x00, 0x00, 0x00, // min Z
            0x00, 0x00, 0x01, 0x00, // max X
            0x00, 0x00, 0x01, 0x00, // max Y
            0x00, 0x00, 0x01, 0x00, // max Z
        ];

        let mut cursor = Cursor::new(&bytes_some[..]);

        let actual = parse_nbt_payload(&mut cursor)?;
        let expected = NBTNode::IntArray {
            name: "BB".to_owned(),
            values: Vec::from([
                0x00_00_00_00,
                0x00_00_00_00,
                0x00_00_00_00,
                0x00_00_01_00,
                0x00_00_01_00,
                0x00_00_01_00,
            ]),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn nbt_long_array_parsing() -> Result<(), NBTParseError> {
        let bytes_some: [u8; _] = [
            0x0c, // Long array tag
            0x00, 0x0b, // name length = 11
            0x4f, 0x43, 0x45, 0x41, 0x4e, //
            0x5f, 0x46, 0x4c, 0x4f, 0x4f, 0x52, // name = OCEAN_FLOOR
            0x00, 0x00, 0x00, 0x25, // payload size = 37
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 1
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 2
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 3
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 4
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 5
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 6
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 7
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 8
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 9
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 10
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 11
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 12
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 13
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 14
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 15
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 16
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 17
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 18
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 19
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 20
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 21
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 22
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 23
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 24
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 25
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 26
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 27
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 28
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 29
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 30
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 31
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 32
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 33
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 34
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 35
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 36
            0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, // Long payload 37
        ];

        let mut cursor = Cursor::new(&bytes_some[..]);

        let actual = parse_nbt_payload(&mut cursor)?;
        let expected = NBTNode::LongArray {
            name: "OCEAN_FLOOR".to_owned(),
            values: Vec::from([0x0080_4020_1008_0402; 37]),
        };

        assert_eq!(actual, expected);

        Ok(())
    }
}
