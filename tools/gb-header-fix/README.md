# gb-header-fix

Patch Game Boy ROM headers and checksums from a `header.toml` configuration file.

## Usage

```
gb-header-fix <rom-file> <header.toml>
```

Modifies the ROM file in-place:
- Writes the Nintendo logo (required for boot ROM validation)
- Sets header fields from `header.toml`
- Pads ROM to next power-of-two size (minimum 32KB)
- Auto-calculates ROM size code from padded size
- Computes and writes header checksum and global checksum

## header.toml

Only `title` is required. All other fields have sensible defaults.

```toml
title = "MYGAME"
cgb_flag = "Hybrid"     # "None" (default), "Hybrid", "CgbOnly"
sgb_flag = false
cartridge_type = 0x00   # ROM ONLY
ram_size = 0x00
destination = "worldwide"  # "japan" or "worldwide"
old_licensee_code = 0x00
# new_licensee_code = "01"  # sets old_licensee_code to 0x33
version = 0x00
```

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `title` | string | (required) | ROM title (max 15 chars for CGB, 16 for DMG) |
| `cgb_flag` | enum | `"None"` | `"None"` = DMG only, `"Hybrid"` = DMG+CGB, `"CgbOnly"` = CGB only |
| `sgb_flag` | bool | `false` | Super Game Boy support |
| `cartridge_type` | hex/int | `0x00` | MBC type (see `header.example.toml` for full list) |
| `ram_size` | hex/int | `0x00` | External RAM size (0x00=None, 0x01=2KB, 0x02=8KB, 0x03=32KB) |
| `destination` | enum | `"worldwide"` | `"japan"` or `"worldwide"` |
| `old_licensee_code` | hex/int | `0x00` | Old licensee code |
| `new_licensee_code` | string | (none) | 2 ASCII chars, automatically sets old_licensee_code to 0x33 |
| `version` | int | `0x00` | Mask ROM version |

### Auto-calculated fields

These are not in `header.toml` — they are computed automatically:

| Offset | Field | How |
|---|---|---|
| `0x0104-0x0133` | Nintendo logo | Fixed 48-byte sequence |
| `0x0148` | ROM size | Derived from padded ROM size |
| `0x014D` | Header checksum | Complement sum of bytes 0x0134-0x014C |
| `0x014E-0x014F` | Global checksum | Sum of all ROM bytes |

## Example

```
gb-header-fix target/mygame.gb header.toml
# Fixed: target/mygame.gb (32KB, header checksum: 0xFF, global checksum: 0x50B5)
```

See `header.example.toml` for a fully commented template.
