# Rust-GB
Compile Rust code to GBZ80 (Work in Progress)  
You can find ROM builds of examples in [release](https://github.com/zlfn/rust-gb/releases/tag/v0.0.1-alpha)

![image](https://github.com/user-attachments/assets/90c049f7-7317-44c9-9c73-a7865c78b24e)
^ `filltest` example of GBDK-2020, ported to Rust.


## How is this possible?
GameBoy is not a possible target of Rust (even its not in [Tier 3](https://doc.rust-lang.org/nightly/rustc/platform-support.html)), and there is currently no suitable (stable) LLVM backend for the CPU in GameBoy. Therefore, the Rust code is compiled using the following process.
```
                         Fake dependencies                      C files                 SM83          
rust-deps                (not acutally linked)                  for SDCC                Assembly files
┌─────────┐              ┌────────────────┐                     ┌──────────┐   SDCC     ┌───────────┐ 
│         │  build-std   │                │                     │ rom0.c   ├───────────►│ rom0.asm  │ 
│  core   ├─────────────►│ libcore*.rlib  │ Compiled LLVM-IR    │ rom1.c   │            │ rom1.asm  │ 
│         │+emit=llvm-ir │ libgbdk*.so    │┌───────────────┐    └──────────┘            └┬──────────┘ 
└─────────┘              │                ││ libcore*.ll   │         ▲                   │            
┌──────────┐             └─────┬──────────┘└──────────┬────┘         │ Treesitter        │ GBDK chain 
│          │              ▲    │                      │              │                   │ - lcc      
│ gbdk-lib │    clang     │    │                      │ llvm-cbe                         │ - sdasgb   
│          ├──────────────┘    │         LLVM-IR file ├──────────► C file           link │ - bankpack 
└──────────┘                   │                      │ llvm-link  for Clang        ┌───►│ - sdlgdb   
  ▲                            │                      │                             │    │ - ihxcheck 
  │ extern                     │           ┌──────────┴────┐   ┌────────────────┐   │    │ - makebin  
┌─┴────────┐                   │           │               │   │ memory.c       │   │    │            
│          │   cargo build     ▼           │ project*.ll   │   │ custom asms    ├───┘    ▼            
│ rust-gb  ├──────────────────────────────►│ crates*.ll    │   │ GBDK c files   │                     
│ source   │                               │               │   │ GBDK asm files │     game.gb         
│          │                               └───────────────┘   └────────────────┘                     
└──────────┘                                                   Real dependencies                                                  
```
1. The Rust compiler can generate LLVM-IR for the ATMega328 processor. (which powers Arduino)
2. LLVM-IR can be converted to C code using [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe).
3. The C code can then be compiled to Z80 Assembly using [sdcc](https://sdcc.sourceforge.net/).
4. Z80 Assembly can be assembled into GBZ80 object code with [sdasgb](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_supported_consoles.html).
5. The GBZ80 object code can be linked with GBDK libraries and built into a Game Boy ROM using [lcc](https://gbdk-2020.github.io/gbdk-2020/docs/api/docs_toolchain.html#lcc).

I referred to [z80-babel](https://github.com/MartinezTorres/z80_babel) for steps 1–3, and used [gbdk-2020](https://github.com/gbdk-2020/gbdk-2020) for steps 4–5.

In the long run, I hope to write LLVM backend for z80 (sm83), and include it in Rust's Tier 3 list. This will dramatically simplify the build chain.

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
* rust (nightly)
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
2. Install Rust toolchains. Because of the llvm-cbe's LLVM version, rustc `nightly` version required.  
  I highly recommand to use [mise](https://github.com/jdx/mise) because the required version of rust nightly may change.  
  But there's no big problem if you install rust the way you want it.
```bash
# Install mise
curl https://mise.run | sh
echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc

# Install rust using mise (Shell restart may be required)
mise install
```
3. Install all dependencies in your linux (Use WSL for Windows)
```bash
# Arch Linux
sudo pacman -S avr-gcc avr-libc sdcc
# Ubuntu
sudo apt install gcc-avr avr-libc sdcc
```
4. Download llvm-cbe binary and build rust build dependencies.
```bash
# This shell script will download llvm-cbe binary from server and build rust dependencies in ext/rust-deps

curl https://mirror.zlfn.space/rust-gb/setup.sh | sh
```
5. Now, `cargo build-rom` (where the `./src` directory is located) will build your GB ROM to `./out/main.gb`

```bash
# Caution: `build-rom` should be executed in the /source directory, not the project root directory
cd ./source

# Build GB ROM from Rust code
cargo build-rom
```

## Related & Similar projects
- [GBDK-2020](https://github.com/gbdk-2020/gbdk-2020) : Provides the library for GameBoy.
- [llvm-cbe](https://github.com/JuliaHubOSS/llvm-cbe) : Compile Rust code to C.
- [z80_babel](https://github.com/MartinezTorres/z80_babel) : Giving an idea to compile Rust code into Z80.
- [gba](https://github.com/rust-console/gba) : Compiles the Rust code into the GameBoy but its Advance. (Unlike DMG, GBA is Rust's Tier 3 target.)

