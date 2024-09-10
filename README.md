# Rust-GB
Compile a Rust code to GBZ80 (WIP)

## How can is it possible?
1. Rust compiler can produce llvm-ir
2. llvm-ir can be compiled to C code ([llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe))
3. C code can be compiled to Z80 Assembly ([sdcc](https://sdcc.sourceforge.net/))
4. Z80 ASM can be assembled for GBZ80 ([sdasgb](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_supported_consoles.html))

I will refer [z80-babel](https://github.com/MartinezTorres/z80_babel) for task 1~3.

## Goal
My goal is to write a Gameboy Develpment Kit to make a gameboy game with Rust. (like [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020/tree/develop))  
If possible, I would like to create a library in pure Rust. But it might end up wrapping gbdk.

Either way, it's going to be quite a long time for me.  
But Gameboy is already 30 years old, and a few more years won't change much.
