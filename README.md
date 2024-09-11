# Rust-GB
Compile Rust code to GBZ80 (Work in Progress)

## How is this possible?
1. The Rust compiler can generate LLVM-IR for the ATMega328 processor. (which powers Arduino)
2. LLVM-IR can be converted to C code using [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe).
3. The C code can then be compiled to Z80 Assembly using [sdcc](https://sdcc.sourceforge.net/).
4. Z80 Assembly can be assembled into GBZ80 object code with [sdasgb](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_supported_consoles.html).
5. The GBZ80 object code can be linked with GBDK libraries and built into a Game Boy ROM using [lcc](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_toolchain.html#lcc).

I referred to [z80-babel](https://github.com/MartinezTorres/z80_babel) for steps 1–3, and used [gbdk-2020](https://github.com/gbdk-2020/gbdk-2020) for steps 4–5.

## Goal
My goal is to develop a Game Boy Development Kit that enables the creation of Game Boy games using Rust. 

Thanks to GBDK, Z80 Assembly generated from Rust can call GBDK's low-level library functions (such as `delay`, `waitpad`, etc.). 

My task is to wrap these functions in high-level Rust abstractions.

## Dependencies
* rust
* avr-gcc
* avr-libc
* llvm
* llvm-cbe
* sdcc

This project is still a work in progress, and I haven't tested it outside of my development environment.

Dependencies may change as the project evolves.
