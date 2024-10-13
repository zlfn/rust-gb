# Rust-GB
Compile Rust code to GBZ80 (Work in Progress)  
You can find ROM builds of examples in [release](https://github.com/zlfn/rust-gb/releases/tag/v0.0.1-alpha)

![image](https://github.com/user-attachments/assets/90c049f7-7317-44c9-9c73-a7865c78b24e)
^ `filltest` example of GBDK-2020, ported to Rust.


## How is this possible?
1. The Rust compiler can generate LLVM-IR for the ATMega328 processor. (which powers Arduino)
2. LLVM-IR can be converted to C code using [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe).
3. The C code can then be compiled to Z80 Assembly using [sdcc](https://sdcc.sourceforge.net/).
4. Z80 Assembly can be assembled into GBZ80 object code with [sdasgb](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_supported_consoles.html).
5. The GBZ80 object code can be linked with GBDK libraries and built into a Game Boy ROM using [lcc](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_toolchain.html#lcc).

I referred to [z80-babel](https://github.com/MartinezTorres/z80_babel) for steps 1–3, and used [gbdk-2020](https://github.com/gbdk-2020/gbdk-2020) for steps 4–5.

## Why use Rust instead of C or ASM?
1. Rust provides higher-level and better grammer than C.
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
* rust (nightly-2024-02-13)
* avr-gcc
* avr-libc
* sdcc

This project is still a work in progress, and I haven't tested it outside of my development environment.

Dependencies may change as the project evolves.

## Build
I do not recommend that you build this project now. because this is WIP and I'm still testing many things.

But if you want to do it, Here is the description below.

1. Clone this repository
```bash
git clone https://github.com/zlfn/rust-gb
cd rust-gb
```
2. Install Rust toolchains. Because of the llvm-cbe's LLVM version, rustc `nightly-2024-02-13` version required.  
  I highly recommand to use [mise](https://github.com/jdx/mise)
```bash
mise install
```
3. Install all dependencies in your linux (Use WSL for Windows)
```bash
# Arch Linux
sudo pacman -S avr-gcc avr-libc sdcc
# Ubuntu
sudo apt install binutils-avr gcc-avr avr-libc avrdude sdcc
```
4. Download llvm-cbe binary and build rust build dependencies.
```bash
# This shell script will download llvm-cbe binary from my server and build rust dependencies in ext/rust-deps
# Be aware that the file is large (1.7GB after decompress)

curl https://mirror.zlfn.space/rust-gb/setup.sh | sh
```
5. Now, `cargo build-rom` (where the `./src` directory is located) will build your GB ROM to `./out/main.gb`

```bash
# build GB ROM from Rust code
cargo build-rom

# build GB ROM from LLVM-IR
cargo llvm-rom
# ... from C code
cargo c-rom
# ... from ASM code
cargo asm-rom
```

## Build chain Description
### Rust bundling
Rust codes in `./source` bundled in one .rs file by [rust-bundler-cp](https://github.com/Endle/rust-bundler-cp)
### Rust -> LLVM-IR
Bundled Rust code is compiled to target `avr-unknown-gnu-atmega328`.  
This will provide 8-bit compatibility for z80.

### LLVM-IR -> C
LLVM-CBE compile LLVM-IR to C code.  

I'm considering If it can be replaced with [llvm-gbz80](https://github.com/Bevinsky/llvm-gbz80)
### C post-processing
The generated C code is for GCC. Therefore, it goes through post-processing before it is entered into SDCC.

Use the `tree-sitter` and `sed` to parse the C code, and replace or add the required codes.
### C -> ASM
SDCC compile C code for GBZ80 (`sm83`)
### ASM -> ROM
I used GBDK's build chain for this. GBDK's `lcc` link ASM with GBDK libraries and build a complete Gameboy ROM file.

