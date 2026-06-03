# template

A minimal rust-gb project. Copy this directory to start a new Game Boy game.

```sh
cargo make        # build target/template.gb
cargo make run    # build and launch in an emulator
```

Edit `src/main.rs` for code, `header.toml` for the ROM title and cartridge
settings. When you add banked code or data, switch the cartridge type in
`header.toml` to an MBC (e.g. `cartridge_type = 0x1B` for MBC5+RAM+BATTERY);
`gb-bank-pack` and `gb-header-fix` size the ROM automatically.
