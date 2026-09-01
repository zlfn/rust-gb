<div align="center">
  <img align="center" width=85% src="media/rust-gb-logo.jpg"/>
  <br/>
</div>

---

[![Crates.io Version](https://img.shields.io/crates/v/rust-gb?style=for-the-badge&logo=rust&color=dea584&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://crates.io/crates/rust-gb)
[![docs.rs](https://img.shields.io/docsrs/rust-gb?style=for-the-badge&logo=docsdotrs&color=%23000000&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://docs.rs/rust-gb/latest/gb/)
[![Crates.io License](https://img.shields.io/crates/l/rust-gb?style=for-the-badge&logo=opensourceinitiative&logoColor=white&color=3DA639)](https://github.com/zlfn/rust-gb/blob/main/LICENSE)

Compile Rust to Game Boy ROMs.

# The Rust-GB Project

Everything needed to write a Game Boy program in Rust, and to build it into a ROM.

**Rust libraries for writing Game Boy programs**
- [`rust-gb`](crates/gb): the hardware abstraction layer, and the crate to start from.
- [`gb-bank`](crates/gb-bank), [`gb-hram`](crates/gb-hram), [`gb-pak`](crates/gb-pak),
  [`gb-ram-fn`](crates/gb-ram-fn): the pieces behind `rust-gb`
- [`gb-rt`](crates/gb-rt): the runtime. [`rrt0.s`](crates/gb-rt/src/rrt0.s) is the
  startup code a cartridge boots into.
- [`gbdk-sys`](crates/gbdk-sys): Rust bindings to
  [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020).

**Host tools for building ROM**
- [`cargo-gb`](tools/cargo-gb), [`gb-header-fix`](tools/gb-header-fix),
  [`gb-bank-pack`](tools/gb-bank-pack): turn a crate into a ROM.
- [`gb-image-fx`](tools/gb-image-fx): convert images into Game Boy tile data.

**Example programs**
- [`examples/`](examples): example Game Boy ROMs written in Rust.

Building Rust-GB also produced [llvm-z80](https://github.com/zlfn/llvm-z80),
[rust-z80](https://github.com/zlfn/rust) and
[decolorize](https://github.com/zlfn/decolorize).

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

* [rust-z80](https://github.com/zlfn/rust) (nightly Rust fork)

## Related & Similar projects

- [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020) : Provides the basic library for Game Boy.
- [llvm-z80](https://github.com/zlfn/llvm-z80) : The LLVM fork providing the Z80/SM83 backend.
- [z80_babel](https://github.com/MartinezTorres/z80_babel) : Gave the idea of compiling Rust code into Z80 in the early stages of the rust-gb project.
- [gba](https://github.com/rust-console/gba) : Compiles Rust code for the Game Boy Advance. (Unlike DMG, GBA is Rust's Tier 3 target.)
