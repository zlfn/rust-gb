<div align="center">
  <img align="center" width=80% src="media/rust-gb-logo.jpg"/>
  <br/>
</div>

---

[![Crates.io Version](https://img.shields.io/crates/v/rust-gb?style=for-the-badge&logo=rust&color=dea584&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://crates.io/crates/rust-gb)
[![docs.rs](https://img.shields.io/docsrs/rust-gb?style=for-the-badge&logo=docsdotrs&color=%23000000&link=https%3A%2F%2Fdocs.rs%2Frust-gb%2Flatest%2Fgb%2F)](https://docs.rs/rust-gb/latest/gb/)
[![Crates.io License](https://img.shields.io/crates/l/rust-gb?style=for-the-badge&logo=opensourceinitiative&logoColor=white&color=3DA639)](https://github.com/zlfn/rust-gb/blob/main/LICENSE)


Compile Rust code to GBZ80 (Work in Progress)  
You can find ROM builds of examples in [release](https://github.com/zlfn/rust-gb/releases/tag/v0.0.1-alpha). Documentation can be found [here](https://docs.rs/rust-gb/latest/gb/).

## How is this possible?
GameBoy is not a possible target of Rust (even its not in [Tier 3](https://doc.rust-lang.org/nightly/rustc/platform-support.html)), and there is currently no suitable (stable) LLVM backend for the CPU in GameBoy. Therefore, the Rust code is compiled using the following process.
1. The Rust compiler can generate LLVM-IR for the ATMega328 processor. (which powers Arduino)
2. LLVM-IR can be converted to C code using [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe).
3. The C code can then be compiled to Z80 Assembly using [sdcc](https://sdcc.sourceforge.net/).
4. Z80 Assembly can be assembled into GBZ80 object code with [sdasgb](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_supported_consoles.html).
5. The GBZ80 object code can be linked with GBDK libraries and built into a Game Boy ROM using [lcc](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_toolchain.html#lcc).

I referred to [z80-babel](https://github.com/MartinezTorres/z80_babel) for steps 1–3, and used [gbdk-2020](https://github.com/gbdk-2020/gbdk-2020) for steps 4–5.

In the long run, I hope to write LLVM backend for z80 (sm83), and include it in Rust's Tier 3 list. This will dramatically simplify the build chain.

## Why use Rust instead of C or ASM?
1. Rust provides higher-level and better grammar than C.
2. Rust's memory stability and strict types help you avoid to write incorrect code (even on a small device).
3. Putting everything aside, it's fun!

## Goal
This project's goal is to develop a Game Boy Development Kit that enables the creation of Game Boy games using Rust, including *safe* management APIs in Game Boy memory, abstracted functions, and more.

Currently, the dependence on GBDK is large, but we plan to gradually reduce it.

## Support
If you like this project, you can always join our [Discussion](https://github.com/zlfn/rust-gb/discussions)!
Please feel free to share your opinions or ideas.

This project is in its very early stages, and we are still designing many things, so it would be nice to have a variety of ideas.

PRs are always welcome too!

## Dependencies
* rust (nightly)
* avr-gcc
* avr-libc
* sdcc

This project is still a work in progress, and I haven't tested it outside of my development environment.

Dependencies may change as the project evolves.

## Related & Similar projects
- [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020) : Provides the library for GameBoy.
- [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe) : Compile Rust code to C.
- [z80_babel](https://github.com/MartinezTorres/z80_babel) : Giving an idea to compile Rust code into Z80.
- [gba](https://github.com/rust-console/gba) : Compiles the Rust code into the GameBoy but its Advance. (Unlike DMG, GBA is Rust's Tier 3 target.)

