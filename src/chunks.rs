// Chunk format
// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

// Chunk Data and Update Light (clientbound packet)
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light

// Unload Chunk (clientbound packet)
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Unload_Chunk

const CHUNK_LENGTH_X: u16 = 16;
const CHUNK_LENGTH_Z: u16 = 16;

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
    use super::*;
}
