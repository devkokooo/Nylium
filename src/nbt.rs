pub enum NBTTagType {
    /// Used to mark the end of compound tags.
    /// It is always a single byte 0.
    /// It may also be the type of empty List tags.
    TAG_End = 0x00,
    /// 1 byte / 8 bits, signed
    /// Sometimes used for booleans.
    TAG_Byte = 0x01,
    /// 2 bytes / 16 bits, signed, big endian
    TAG_Short = 0x02,
    /// 4 bytes / 32 bits, signed, big endian
    TAG_Int = 0x03,
    /// 8 bytes / 64 bits, signed, big endian
    TAG_Long = 0x04,
    /// 4 bytes / 32 bits, signed, big endian,
    TAG_Float = 0x05,
    /// 8 bytes / 64 bits, signed, big endian
    TAG_Double = 0x06,
    /// 4 bytes / 32 bits, signed, big endian for size, then bytes of length size
    TAG_Byte_Array = 0x07,
    /// 2 bytes / 16 bits, unsigned, big endian for size, then bytes of length size as UTF-8
    /// no null termination
    TAG_String = 0x08,
    /// 1 byte / 8 bits for tag ID of the list's contents, then
    /// 4 bytes / 32 bits, big endian for size, then
    /// length size number of items of tag ID
    TAG_List = 0x09,
    /// Contains any number of tags, delimited by TAG_End.
    /// Each tag consisting of 1 byte / 8 bits tag ID, followed by
    /// 2 bytes / 16 bits, unsigned, big endian for size, then
    /// UTF-8 formatted string containing the tag name.
    /// Payload data is last.
    ///
    /// Although there is no hard limit to the number of tags
    /// within a Compount, note that Compount and List tags
    /// maybe not be nested beyond a depth of 512.
    TAG_Compound = 0x0A,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of TAG_Int payloads.
    TAG_Int_Array = 0x0B,
    /// 4 bytes / 32 bits, signed, big endian for size, then
    /// size number of TAG_Long payloads.
    TAG_Long_Array = 0x0C,
}

impl TryFrom<u8> for NBTTagType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::TAG_End),
            0x01 => Ok(Self::TAG_Byte),
            0x02 => Ok(Self::TAG_Short),
            0x03 => Ok(Self::TAG_Int),
            0x04 => Ok(Self::TAG_Long),
            0x05 => Ok(Self::TAG_Float),
            0x06 => Ok(Self::TAG_Double),
            0x07 => Ok(Self::TAG_Byte_Array),
            0x08 => Ok(Self::TAG_String),
            0x09 => Ok(Self::TAG_List),
            0x0A => Ok(Self::TAG_Compound),
            0x0B => Ok(Self::TAG_Int_Array),
            0x0C => Ok(Self::TAG_Long_Array),
            other => Err(other),
        }
    }
}

pub struct NBTTree {}

#[derive(Debug)]
pub enum NBTParseError {}

pub fn parse_nbt_payload(payload: &[u8]) -> Result<NBTTree, NBTParseError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_tag_parsing() -> Result<(), NBTParseError> {
        // Expected should be:
        // root NBTCompound { NBTString Status = "minecraft:full" }
        let nbt_data = [
            0x0a, 0x00, 0x00, 0x08, 0x00, 0x06, 0x53, 0x74, 0x61, 0x74, 0x75, 0x73, 0x00, 0x0e,
            0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x66, 0x75, 0x6c, 0x6c,
            0x03,
        ];

        let actual = parse_nbt_payload(&nbt_data)?;

        Ok(())
    }
}
