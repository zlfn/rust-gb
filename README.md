<div align="center">
  <img align="center" width=80% src="media/rust-gb-logo.jpg"/>
  <br/>
</div>

---

[![Crates.io Version](https://img.shields.io/crates/v/rust-gb?style=for-the-badge&logo=rust&color=dea584&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://crates.io/crates/rust-gb)
[![docs.rs](https://img.shields.io/docsrs/rust-gb?style=for-the-badge&logo=docsdotrs&color=%23000000&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://docs.rs/rust-gb/latest/gb/)
[![Crates.io License](https://img.shields.io/crates/l/rust-gb?style=for-the-badge&logo=opensourceinitiative&logoColor=white&color=3DA639)](https://github.com/zlfn/rust-gb/blob/main/LICENSE)

Compile Rust to Game Boy ROMs.

## What's in here

**Game Boy Rust libraries**
- `gbdk-sys`: Rust bindings to the GBDK-2020 runtime (graphics, sound, input, console).
- `gb-bank`: a compile-time-safe ROM bank switching API. GhostCell-style tokens make holding a pointer into an unmapped bank a compile error; a `#[bank]` function becomes a `Warp` (the `Future` analog of a banked call) that you run with `.drive()`.

**Build tool**
- `cargo-gb`: the `cargo gb` subcommand that turns a crate into a ROM (`cargo gb build`). It compiles the staticlib, bin-packs banked code into 16 KiB banks, links with the toolchain's bundled SM83 linker, and writes the cartridge header. Everything comes from the rust-z80 sysroot, so no tool or linker paths need configuring.

**Utilities**
- `gb-image-fx`: convert images (PNG, JPEG, BMP, ...) to Game Boy / Game Boy Color tile data.

**Examples**
- `examples/`: runnable demos. Start from `template` (a minimal project to copy), and see `bank-test` and a set of ported GBDK examples.

## Building a ROM

Install the build tool once:

```sh
cargo install --path tools/cargo-gb
```

Then, in any example (or your own crate):

```sh
cargo gb build    # compiles and writes target/<name>.gb
cargo gb run      # build, then launch $EMULATOR (default: sameboy)
```

The rust-z80 toolchain bundles the SM83 linker and LLVM tools, so there is nothing else to configure.

## How is this possible?

The Game Boy is not a Rust target (it is not even in [Tier 3](https://doc.rust-lang.org/nightly/rustc/platform-support.html)), 
and there has been no stable LLVM backend for its CPU. 

Rust-GB uses a custom LLVM backend for Game Boy:

1. [rust-z80](https://github.com/zlfn/rust), a fork of the Rust compiler, targets `sm83` and emits LLVM-IR.
2. [llvm-z80](https://github.com/zlfn/llvm-z80), an LLVM fork with a Z80/SM83 backend, lowers that IR to SM83 machine code.
3. The objects are linked against the runtime and built into a Game Boy ROM.
   `cargo gb build` ([`cargo-gb`](tools/cargo-gb)) drives the whole pipeline.

## Why use Rust instead of C or ASM?

1. Rust provides a higher-level and better grammar than C.
2. Rust's memory stability and strict types help you avoid writing incorrect code (even on a small device).
3. Putting everything aside, it's fun!

## Goal

This project's goal is to develop a Game Boy Development Kit that 
enables the creation of Game Boy games using Rust, 
including *safe* management APIs in Game Boy memory, abstracted functions, and more.

## Support

If you like this project, you can always join our [Discussion](https://github.com/zlfn/rust-gb/discussions)!
Please feel free to share your opinions or ideas.

This project is in its very early stages, and we are still designing many things, 
so it would be nice to have a variety of ideas.

PRs are always welcome too!

## Dependencies

* [rust-z80](https://github.com/zlfn/rust) (nightly Rust fork, built with in-tree `lld`)
* [llvm-z80](https://github.com/zlfn/llvm-z80) (the SM83 LLVM backend, built into rust-z80)

This project is still a work in progress, and I haven't tested it outside of my development environment. 
Dependencies may change as the project evolves.

## Related & Similar projects

- [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020) : Provides the basic library for Game Boy.
- [llvm-z80](https://github.com/zlfn/llvm-z80) : The LLVM fork providing the Z80/SM83 backend.
- [z80_babel](https://github.com/MartinezTorres/z80_babel) : Gave the idea of compiling Rust code into Z80 in the early stages of the rust-gb project.
- [gba](https://github.com/rust-console/gba) : Compiles Rust code for the Game Boy Advance. (Unlike DMG, GBA is Rust's Tier 3 target.)
