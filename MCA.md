# Anvil file format (Java Edition 26.3)

Anvil is the file storage format for Java Edition chunks. It was implemented in snapshot 12w07a for release in Java Edition 1.2.1. `level.dat` `TAG_Int("version")` is **19133**.

Anvil reuses the **Region file format** container (sectors, location table, timestamps, payload framing). The only format changes from MCRegion were the file extension (`.mcr` → `.mca`), the `level.dat` version, and the **chunk NBT** layout described below. Later container features (LZ4, custom compression, external `.mcc` overflow) remain current in 26.3.

This document combines the [Region file format](https://minecraft.wiki/w/Region_file_format) and [Anvil file format](https://minecraft.wiki/w/Anvil_file_format) wiki specs into one implementable description. Chunk NBT has evolved further since Anvil’s introduction (Flattening, paletted storage, 1.18+ height, separate entity/POI regions, etc.); see [Chunk format](https://minecraft.wiki/w/Chunk_format) for the modern schema. The **container** below is what JE 26.3 still reads and writes for `.mca` files.

---

## Location

Region files are found in the Java Edition world directory in the following directories:

- `directory/entities`
- `directory/poi`
- `directory/region`

Where `directory` may refer to the Minecraft world save directory.

Region files take the form:

```text
r.<x>.<z>.mca
```

(Legacy MCRegion used the same naming with `.mcr`. Vanilla Anvil worlds use `.mca` only for active region data.)

The X and Z position of a region file is determined through the floor of dividing the chunk coordinates by 32. For example, a chunk at `(30, -3)` would be in the region `(0, -1)`, and the chunk at `(1500, -600)` would be at `(46, -19)`.

Written in code, this is more commonly achieved through performing an **arithmetic shift** 5 bits to the right. Note that an arithmetic shift is different from a logical shift. Most programming languages will perform this when using a signed integer.

Getting the block position uses **512** instead of 32 (`32 × 16`), and when shifting uses **9** instead of 5 (`log2(512)`).

Why bit shifting is used over standard division, even with integers: integer division floors only if the integer is positive. If the integer is negative (about half the time), the result is rounded up instead of down.

A region stores **32×32** chunks (**1024** chunks), covering **512×512** blocks.

### Example

| | |
| --- | --- |
| Block pos | `(0, 0, 0)` |
| Chunk pos | `(0, 0)` |
| Region file | `r.0.0.mca` |

---

## Structure

A region file is built out of **sectors**. These sectors are **4 KiB** large (equal to the common memory page size for operating system kernels). The entire file is sector-aligned, and the total size should thus be a multiple of 4 KiB. If the file isn't aligned to this boundary there is a possibility that it is corrupt.

The file does **not** begin with a magic number; it begins directly with the header.

The file has a header of two tables containing the chunk data. Sectors following these tables may be allocated to a chunk. When a chunk is newly created, or grows in sectors, the code will look for this number of sequential sectors that are free. If it fails to find this, the chunk will be appended to the end of the file.

| Sector | Description |
| ------ | ----------- |
| 0, 1 | Header, contains payload metadata |
| 2… | Payload data and unused sectors |

### Header

The first two sectors of the file contain the two headers, occupying one sector each. The first table contains the position data of the chunks in the region file itself; the second provides timestamps for the last update of said chunks.

These tables compose a **1024**-element list (`32²`) of big-endian 32-bit integers. Each chunk has a corresponding X and Z position. This is computed through floored division and taking the modulo of the index, here represented as `i`:

```text
x = i mod 32
z = floor(i / 32)
```

With the reverse:

```text
i = x + 32 * z
```

Ensuring X and Z are constrained to a value from `0..31` (inclusive), i.e. the result of a modulo of 32.

To read the position data out of the first table, read bytes `4*i` to `4*i+4` (exclusive). The same can be done for the timestamp data, but adding **4096** to both offsets (the second sector).

The location entry uses a specific format: the first **3** bytes are the **offset in sectors** from the start of the file; the last (least significant) byte is the **size in sectors** of the chunk's data.

```text
offset_sectors = (entry >> 8) & 0xFFFFFF
size_sectors   = entry & 0xFF
```

This implicitly means that the maximum number of sectors for a chunk is **255**, and thus a non-external chunk can be at most `4 KiB × 255 = 1020 KiB` in size.

| Byte | 0 | 1 | 2 | 3 |
| ---- | - | - | - | - |
| Description | Offset | | | Length |

- Entry value **0** means the chunk is absent / empty (not present in the file).
- A present chunk’s `offset_sectors` is at least **2** (payload starts after the two header sectors).

### Payload

Chunk data begins with a (big-endian) four-byte signed **length** field that indicates the exact length of the remaining chunk data in bytes. The following byte indicates the **compression scheme** used for chunk data, and the remaining `(length - 1)` bytes are the compressed chunk data.

Minecraft always pads the last chunk's data to be a multiple-of-4096B in length (so that the entire file has a size that is a multiple of 4 KiB). Minecraft does not accept files in which the last chunk is not padded. Note that this padding is **not** included in the length field.

| Byte | 0 | 1 | 2 | 3 | 4 | 5… |
| ---- | - | - | - | - | - | -- |
| Description | length (in bytes) | | | | compression type | compressed data (`length - 1` bytes) |

There are currently five defined compression schemes:

| Value | Method |
| ----- | ------ |
| 1 | GZip (RFC 1952) (unused in practice) |
| 2 | Zlib (RFC 1950) — **vanilla default** when saving |
| 3 | Uncompressed (since a version before 1.15.1) |
| 4 | LZ4 (since 24w04a; enabled in `server.properties`) |
| 127 | Custom compression algorithm (since 24w05a, for third-party servers). A namespaced string must follow representing the algorithm used. The string is preceded by its length, encoded as an unsigned 16-bit big-endian integer, then the compressed payload. |

The uncompressed data is in **NBT** format and follows the chunk format (see below for the Anvil introduction layout; modern saves use later schemas). Chunks are always saved using compression scheme **2** by the official client unless another option is enabled.

#### External chunk files (`.mcc`)

If the compressed chunk data would exceed **1020 KiB** (255 sectors):

1. The compression scheme's value is increased by **128** (bit 7 set).
2. The length is set to **1**.
3. The compressed chunk data is instead saved in a file called:

```text
c.<x>.<z>.mcc
```

where `x` and `z` are the chunk's absolute coordinates, in the same directory as the region file.

Readers must check bit 7 of the compression byte; if set, load from the `.mcc` file and use `(type & 0x7F)` as the real scheme.

---

## Chunk format (Anvil)

Default Overworld chunks cover **16×384×16** blocks (buildable Y **-64**…**319**). The Nether and the End remain **16×256×16** by default. Since 1.18, **all** sections spanning the world’s height are present in the save, including empty (air-filled) ones.

Block states and biomes use **paletted containers** (Flattening onward; tag layout stabilized in 1.18). Numeric block IDs / `"Add"` layers from Anvil’s introduction are gone.

Block array order is **YZX** (for better compression). Biomes are saved per section in **4×4×4** cells (so tools can edit them, and biome shifts after generation changes are avoided).

### Sections

- A `"sections"` **List** of **Compound** tags holds the chunk’s section data (**24** compounds for a default Overworld chunk; **16** in Nether/End). Root `"yPos"` is the lowest section Y (e.g. **-4** in 1.18+ Overworld).
- Each section is `16×16×16` and stores blocks in a `"block_states"` paletted container, plus optional `"SkyLight"` and `"BlockLight"` nibble arrays (2048 bytes each when present).
- Each section has a `"Y"` **Byte** tag saying which section it is. In a default Overworld chunk, **-4** is at the bottom and **19** is the topmost section (**0**…**15** in Nether/End).
- Each section also has a `"biomes"` paletted container (see below).

### Paletted containers

A paletted container is a `"palette"` list plus an optional packed `"data"` **LongArray**. Indices in `"data"` point into `"palette"`. If the palette has only one entry, `"data"` is omitted and that entry fills the whole volume.

**`block_states`** (4096 indices — one per block in the section):

- `"palette"` entries are block states (`"Name"` / properties, or a bare resource-location string for the default state). Vanilla writes at most **4096** entries (≤ **12** bits per index) but can read larger palettes.
- Bit length per index = `ceil(log2(palette_len))`, then raised to a **minimum of 4** bits.
- Indices are packed into 64-bit longs **without** straddling long boundaries: if the next index does not fit in the remaining bits of the current long, it starts at bit 0 of the next long. Bit length may differ per section.

**`biomes`** (64 indices — one per 4×4×4 cell in the section; JE 26.3):

- `"palette"` entries are biome resource-location strings. Vanilla writes at most **64** entries (≤ **6** bits per index) but can read larger palettes.
- Bit length = `ceil(log2(palette_len))` with **no** 4-bit minimum. Same non-straddling long packing as block states.
- (Upcoming JE 26.4 stores 4096 biome indices per section instead of 64.)

### Array indexing

Index within a section’s `16×16×16` block arrays:

```text
index = (y * 16 + z) * 16 + x    // YZX
```

Biome cell index over the section’s 4×4×4 grid (`x,z,y` in `0..3`):

```text
biome_index = (y * 4 + z) * 4 + x    // YZX
```

### Biomes and Heightmaps

- Biomes live in each section’s `"biomes"` paletted container (above), not a chunk-level `"Biomes"` byte array.
- `"Heightmaps"` is a **Compound** of named **LongArray**s (`MOTION_BLOCKING`, `MOTION_BLOCKING_NO_LEAVES`, `OCEAN_FLOOR`, `OCEAN_FLOOR_WG`, `WORLD_SURFACE`, `WORLD_SURFACE_WG`). Each map packs **256** values at **9** bits each into **37** longs (7 values per long; high bit unused). Values are unsigned heights **above the bottom of the world** (0…384 inclusive for default Overworld); converting a pre-1.18 world adds **64** to every value.

Additional / current tag details: [Chunk format](https://minecraft.wiki/w/Chunk_format).

---

## level.dat

For Anvil worlds, `level.dat` includes `TAG_Int("version")` set to **19133**.

On historical conversion from MCRegion, the game kept old `.mcr` files and backed up `level.dat` as `level.dat_mcr` for compatibility. Those `.mcr` files are never used again by Anvil; they may be deleted to shrink backups.

---

## History (container-relevant)

| Version | Change |
| ------- | ------ |
| 12w07a (1.2.1) | Anvil introduced (`.mca`, chunk sections, `version` 19133) |
| 17w47a (1.13) | Flattening; sections move toward paletted block storage |
| 20w14a (1.16) | Region files opened in synchronous mode to reduce crash corruption |
| 21w37a (1.18) | Overworld height **-64**…**319** (384); `yPos` / full `sections` list; per-section biomes |
| 24w04a (1.20.5) | LZ4 compression option (`server.properties`) |
| 24w05a | Custom compression type `127` for third-party servers |

---

## Quick reference

```text
.mca file
├── [0, 4096)      locations[1024]   // BE u32: (offset<<8)|size
├── [4096, 8192)   timestamps[1024]  // BE u32 Unix seconds
└── [8192, EOF)    payloads + free sectors (4 KiB aligned)

payload @ offset*4096
├── BE i32 length
├── u8 compression   // +128 ⇒ data in c.x.z.mcc
└── length-1 bytes   // compressed NBT (or algorithm name + data if type 127)
```
