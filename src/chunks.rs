// Chunk format
// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

// Chunk Data and Update Light (clientbound packet)
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light

// Unload Chunk (clientbound packet)
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Unload_Chunk

use thiserror::Error;

const CHUNK_LENGTH: u8 = 16;

// total height = 384; 384 / 16 = 24 sections
struct ChunkColumn {
    sections: Vec<ChunkSection>,
}

struct ChunkSection {}

// Servers do not need to implement the local palette initially
// (instead always using 15 bits per block). It's a good
// optimization path for later.

// ProtoChunk - a chunk still being generated or prepared.
// It holds intermediate terrain and generation data and
// advances through chunk-generation stages before becoming a
// fully usable world chunk.

// LevelChunk - a fully generated, live chunk in the world.
// It is the normal representation for loaded chunks that the
// server can simulate, update, and send to clients.

/// Region length in chunks.
/// Regions store 32x32 chunks (1024 chunks).
const REGION_LENGTH: u8 = 32;

/// Region files are built out of sectors. These sectors are
/// 4 KiB (equal to common memory page size for OS kernels).
/// The entire file is sector-aligned, and the total size should
/// be a multiple of 4 KiB. If the file isn't aligned to this
/// boundary, there is a possibility that it is corrupt.
///
/// The file does **not** begin with a magic number, but begins
/// directly with the header.
const REGION_SECTOR_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionType {
    GZip = 1,
    Zlib = 2,
    Uncompressed = 3,
    LZ4 = 4,
    Custom = 127,
}

impl TryFrom<u8> for CompressionType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::GZip),
            0x02 => Ok(Self::Zlib),
            0x03 => Ok(Self::Uncompressed),
            0x04 => Ok(Self::LZ4),
            0x7F => Ok(Self::Custom),
            other => Err(other),
        }
    }
}

#[derive(Debug, Error)]
#[error("error parsing chunks from region file")]
pub enum ChunkParsingError {
    #[error("region file - IO error")]
    Io(#[from] std::io::Error),

    #[error("tryfrom casting error")]
    TryFrom(#[from] std::array::TryFromSliceError),

    #[error("unknown compression type: {0}")]
    UnknownCompressionType(u8),
}

impl From<u8> for ChunkParsingError {
    fn from(value: u8) -> Self {
        Self::UnknownCompressionType(value)
    }
}

// TODO: FOCUS ON BLOCK-STATE CONTAINER TESTING
// 1. Block-only model - match coordinate order to protocol data-array

/// TODO: Data-array Format
/// The Data Array stores entries as Bits Per Entry -- bit integers,
/// corresponding to either local or global palette indices depending on
/// the palette format in use. If Bits Per Entry is 0, it is empty.
///
/// 💡 Entries are stored in order of increasing x coordinate,
/// within rows at increasing z coordinates, within layers at
/// increasing y coordinates. In other words, the x coordinate
/// increases the fastest, and the y coordinate the slowest.
///
///
#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read, Seek, SeekFrom},
    };

    use flate2::{Compression, bufread::ZlibDecoder};

    use super::*;

    #[test]
    fn region_file_chunk_parsing() -> Result<(), ChunkParsingError> {
        let file = File::open("test_data/chunks/r.0.0.mca")?;
        let mut buf_reader = BufReader::new(file);

        // FIRST sector = table of 1024-element list of BE 32-bit ints
        // position data of chunks in region
        // each chunk has X and Z position, computed through floor division and mod index
        // x = i mod 32
        // z = floor(i / 32)
        // reverse is i = x + 32 * z
        let mut sector_one = [0_u8; REGION_SECTOR_SIZE];
        buf_reader.read_exact(&mut sector_one)?;
        let mut locations = [0_u32; 1024];

        // SECOND sector = timestamps in UNIX seconds for the last update of each chunk
        let mut sector_two = [0_u8; REGION_SECTOR_SIZE];
        buf_reader.read_exact(&mut sector_two)?;
        let mut timestamps = [0_u32; 1024];

        let mut i = 0;

        while i < REGION_SECTOR_SIZE {
            let location_entry = &sector_one[i..i + 4];
            let timestamp_entry = &sector_two[i..i + 4];

            let loc32 = u32::from_be_bytes(<[u8; 4]>::try_from(location_entry)?);
            let time32 = u32::from_be_bytes(<[u8; 4]>::try_from(timestamp_entry)?);

            locations[i / 4] = loc32;
            timestamps[i / 4] = time32;
            i += 4;
        }

        // Iterate through 1024 locations & timestamps
        for i in 0..1024 {
            let chunk_x = i % REGION_LENGTH as usize;
            let chunk_z = i / REGION_LENGTH as usize;

            let location = locations[i];

            if location == 0x0 {
                continue;
            }

            let sector_offset = (location >> 8) & 0xFF_FFFF;
            let sector_size = location & 0xFF;

            let timestamp = timestamps[i];

            // PARSING CHUNKS
            // seek from start with sector_offset * REGION_SECTOR_SIZE
            // offset:2 = 2 * 4096 = 8192
            let byte_offset = u64::from(sector_offset) * REGION_SECTOR_SIZE as u64;
            buf_reader.seek(SeekFrom::Start(byte_offset))?;

            // payload length refers to compressed data's length
            let mut payload_len = [0u8; 4];
            buf_reader.read_exact(&mut payload_len)?;
            let payload_len = u32::from_be_bytes(payload_len) as usize;

            let mut compression_type = [0u8];
            buf_reader.read_exact(&mut compression_type)?;
            let compression_type = compression_type[0];

            let mut compressed = vec![0u8; payload_len - 1];
            buf_reader.read_exact(&mut compressed)?;

            // decompress payload based on compression type
            let comp_type = CompressionType::try_from(compression_type)?;

            println!(
                "Chunk (X:{chunk_x}, Z:{chunk_z}) [offset:{sector_offset}, size:{sector_size}, timestamp:{timestamp}]"
            );

            // TODO: treat decompressed chunk data as NBT and parse it
            if comp_type == CompressionType::Zlib {
                let mut zd = ZlibDecoder::new(compressed.as_slice());
                let mut decomp = Vec::new();

                zd.read_to_end(&mut decomp)?;
                println!(
                    "[comp_type:{compression_type}, len_comp:{}, len_decomp:{}]",
                    compressed.len(),
                    decomp.len()
                );

                let mut i = 0;

                // TODO: temp, manual parsing to verify bytes
                while i < 512 {
                    let byte = decomp[i];
                    print!("0x{byte:02x} ");
                    i += 1;
                }
            }
        }

        Ok(())
    }
}
