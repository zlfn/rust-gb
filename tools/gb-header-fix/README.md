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
- Fails if the padded ROM is larger than the cartridge type can reach

## header.toml

Only `title` is required. All other fields have sensible defaults.

```toml
title = "MYGAME"
cgb_flag = "Hybrid"     # "None" (default), "Hybrid", "CgbOnly"
sgb_flag = false
cartridge_type = "ROM"  # or "MBC5", or the header byte 0x19
ram_size = 0x00
destination = "worldwide"  # "japan" or "worldwide"
old_licensee_code = 0x00
# new_licensee_code = "01"  # sets old_licensee_code to 0x33
version = 0x00
# wide_banks = false      # MBC1 banks 32-127, MBC5 banks 256-511
```

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `title` | string | (required) | ROM title (max 15 chars for CGB, 16 for DMG) |
| `cgb_flag` | enum | `"None"` | `"None"` = DMG only, `"Hybrid"` = DMG+CGB, `"CgbOnly"` = CGB only |
| `sgb_flag` | bool | `false` | Super Game Boy support |
| `cartridge_type` | string/int | `"ROM"` | MBC type by name (`"MBC5"`), or the header byte; see `header.example.toml` for the accepted names |
| `ram_size` | hex/int | `0x00` | External RAM size (0x00=None, 0x01=2KB, 0x02=8KB, 0x03=32KB) |
| `destination` | enum | `"worldwide"` | `"japan"` or `"worldwide"` |
| `old_licensee_code` | hex/int | `0x00` | Old licensee code |
| `new_licensee_code` | string | (none) | 2 ASCII chars, automatically sets old_licensee_code to 0x33 |
| `version` | int | `0x00` | Mask ROM version |
| `wide_banks` | bool | `false` | Use the cartridge's second bank register (MBC1 or MBC5 only) |

### Wide banking

A bank number is one byte by default, which reaches bank 31 on MBC1 and bank 255
on MBC5: the width of each cartridge's first bank register. `wide_banks = true`
brings in the second one (`0x4000` on MBC1, `0x3000` on MBC5) and raises those to
127 and 511.

`cargo-gb` reads this field before compiling and passes `--cfg gb_wide_bank` to
`gb-bank`, so changing it rebuilds the program. A wide build cannot link
`gbdk-sys`, whose runtime writes only the first register and keeps a one-byte
shadow of the mapped bank.

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
