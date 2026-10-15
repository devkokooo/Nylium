use thiserror::Error;

pub enum NBTTagType {
    /// Used to mark the end of compound tags.
    /// It is always a single byte 0.
    /// It may also be the type of empty List tags.
    End = 0x00,
    /// 1 byte / 8 bits, signed
    /// Sometimes used for booleans.
    Byte = 0x01,
    /// 2 bytes / 16 bits, signed, big endian
    Short = 0x02,
    /// 4 bytes / 32 bits, signed, big endian
    Int = 0x03,
    /// 8 bytes / 64 bits, signed, big endian
    Long = 0x04,
    /// 4 bytes / 32 bits, signed, big endian,
    Float = 0x05,
    /// 8 bytes / 64 bits, signed, big endian
    Double = 0x06,
    /// 4 bytes / 32 bits, signed, big endian for size, then bytes of length size
    ByteArray = 0x07,
    /// 2 bytes / 16 bits, unsigned, big endian for size, then bytes of length size as UTF-8
    /// no null termination
    String = 0x08,
    /// 1 byte / 8 bits for tag ID of the list's contents, then
    /// 4 bytes / 32 bits, big endian for size, then
    /// length size number of items of tag ID
    List = 0x09,
    /// Contains any number of tags, delimited by `TAG_End`.
    /// Each tag consisting of 1 byte / 8 bits tag ID, followed by
    /// 2 bytes / 16 bits, unsigned, big endian for size, then
    /// UTF-8 formatted string containing the tag name.
    /// Payload data is last.
    ///
    /// [0x0A] [0x00 0x00] [   ] [0x08, 0x00, 0x06]
    /// tag ID    size     name       payload
    ///
    ///                      r    o    o    t
    /// [0x0A] [0x00 0x04] [0x72 0x6F 0x6F 0x74] [0x01 0x00 0x00 0x00 0x00]
    /// tag ID  name size        tag name         payload
    ///
    /// Although there is no hard limit to the number of tags
    /// within a Compound, note that Compound and List tags
    /// maybe not be nested beyond a depth of 512.
    Compound = 0x0A,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of `TAG_Int` payloads.
    IntArray = 0x0B,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of `TAG_Long` payloads.
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
    Compound {
        name: String,
        children: Vec<NBTNode>,
    },
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
    String {
        name: String,
        value: String,
    },
}

#[derive(Debug, Error)]
#[error("NBT parse error")]
pub enum NBTParseError {
    PayloadTooSmall,
    SliceConversion(#[from] std::array::TryFromSliceError),
    TypeConversion(#[from] std::num::TryFromIntError),
    Utf8(#[from] std::str::Utf8Error),

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

pub fn parse_nbt_payload(nbt_data: &[u8]) -> Result<NBTNode, NBTParseError> {
    if nbt_data.len() < 3 {
        return Err(NBTParseError::PayloadTooSmall);
    }

    let tag = nbt_data[0];
    let tag = NBTTagType::try_from(tag)?;

    let name = parse_nbt_name(&nbt_data[1..])?;
    let name_len = name.len();
    println!("{tag}, name={name}, name_len={name_len}");

    // Skip the name parsing, go straight to payload
    if name_len == 0 {
        let rest_of_data = &nbt_data[3..];
        println!("rest={rest_of_data:?}");

        match tag {
            NBTTagType::Compound => {
                let mut children: Vec<NBTNode> = Vec::new();

                let node = parse_nbt_payload(&rest_of_data)?;
                children.push(node);

                let compound = NBTNode::Compound {
                    name: String::new(),
                    children,
                };

                return Ok(compound);
            }
            _ => {}
        }
    } else {
        let rest_of_data = &nbt_data[3 + name_len..];
        println!("rest={rest_of_data:?}");

        match tag {
            NBTTagType::Byte => {
                let payload = rest_of_data[0];
                let payload = i8::try_from(payload)?;

                let byte = NBTNode::Byte {
                    name: name.to_owned(),
                    value: payload,
                };

                return Ok(byte);
            }
            NBTTagType::Int => {
                let payload: [u8; 4] = rest_of_data[0..4].try_into()?;
                let payload = i32::from_be_bytes(payload);

                let int = NBTNode::Int {
                    name: name.to_owned(),
                    value: payload,
                };

                return Ok(int);
            }
            NBTTagType::String => {
                let str_len: [u8; 2] = rest_of_data[0..=1].try_into()?;
                let str_len = u16::from_be_bytes(str_len) as usize;

                let payload = &rest_of_data[2..2 + str_len];
                let payload = str::from_utf8(payload)?.to_owned();

                let string = NBTNode::String {
                    name: name.to_owned(),
                    value: payload,
                };

                return Ok(string);
            }
            _ => {}
        }
    }

    Ok(NBTNode::Byte {
        name: "fake".to_owned(),
        value: 0,
    })
}

/// Parses first 2 bytes (u16) for name length
/// Then parses name length bytes for the name
/// Returns empty string if no name
fn parse_nbt_name(bytes: &[u8]) -> Result<&str, NBTParseError> {
    let name_len: [u8; 2] = bytes[0..=1].try_into()?;
    let name_len = u16::from_be_bytes(name_len) as usize;

    if name_len == 0 {
        return Ok("");
    }

    let name = &bytes[2..2 + name_len];
    let name = std::str::from_utf8(name)?;

    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// `NBTCompound` {
    ///     name: "",
    ///     `NBTString` {
    ///         name: "Status",
    ///         value: "minecraft:full",
    ///     }
    /// }
    #[test]
    fn nbt_compound_parsing() -> Result<(), NBTParseError> {
        // Expected should be:
        // root NBTCompound { NBTString Status = "minecraft:full" }
        let nbt_data = [
            0x0a, // Compound tag
            0x00, 0x00, // name length = 0
            // everything after is payload since no name
            0x08, // String tag
            0x00, 0x06, // name length = 6
            0x53, 0x74, 0x61, 0x74, 0x75, 0x73, // name = Status
            0x00, 0x0e, // payload length = 14
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x66, 0x75, 0x6c,
            0x6c, // minecraft:full
                  // 0x03, // Int tag
                  // 0x00, 0x04, // name length = 4
                  // 0x7a, 0x50, 0x6f, 0x73, // name = zPos
                  // 0x00, 0x00, 0x00, 0x00, // value = 0
        ];

        let actual = parse_nbt_payload(&nbt_data)?;
        let expected = NBTNode::Compound {
            name: String::new(),
            children: Vec::from([
                NBTNode::String {
                    name: "Status".to_owned(),
                    value: "minecraft:full".to_owned(),
                },
                // NBTNode::Int {
                //     name: "zPos".to_owned(),
                //     value: 0,
                // },
            ]),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn nbt_byte_parsing() -> Result<(), NBTParseError> {
        // Y = 64
        let byte_data = [
            0x01, // tag ID
            0x00, 0x01, // name size - u16
            0x59, // name (Y) - UTF-8 bytes
            0x40, // payload (byte, 64) - 1 byte, signed
        ];

        // Mob = true
        let bool_data = [
            0x01, // tag ID
            0x00, 0x03, // name size - u16
            0x4D, 0x6F, 0x62, // name (Mob) - UTF-8 bytes
            0x01, // payload (bool, true) - 1 byte, signed
        ];

        let actual_byte = parse_nbt_payload(&byte_data)?;
        let actual_bool = parse_nbt_payload(&bool_data)?;

        let expected_byte = NBTNode::Byte {
            name: "Y".to_owned(),
            value: 64,
        };
        let expected_bool = NBTNode::Byte {
            name: "Mob".to_owned(),
            value: 1, // 1 for true, 0 for false
        };

        assert_eq!(actual_byte, expected_byte);
        assert_eq!(actual_bool, expected_bool);

        Ok(())
    }

    #[test]
    fn nbt_string_parsing() -> Result<(), NBTParseError> {
        let bytes = [
            0x08, // String tag
            0x00, 0x06, // name length = 6
            0x53, 0x74, 0x61, 0x74, 0x75, 0x73, // name = Status
            0x00, 0x0e, // payload length = 14
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x66, 0x75, 0x6c,
            0x6c, // minecraft:full
        ];

        let actual = parse_nbt_payload(&bytes)?;
        let expected = NBTNode::String {
            name: "Status".to_owned(),
            value: "minecraft:full".to_owned(),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn nbt_int_parsing() -> Result<(), NBTParseError> {
        let bytes = [
            0x03, // Int tag
            0x00, 0x04, // name length = 4
            0x7a, 0x50, 0x6f, 0x73, // name = zPos
            0x00, 0x00, 0x00, 0x00, // value = 0
        ];

        let actual = parse_nbt_payload(&bytes)?;
        let expected = NBTNode::Int {
            name: "zPos".to_owned(),
            value: 0,
        };

        assert_eq!(actual, expected);

        Ok(())
    }
}
