//! # A crate for GameBoy (Color) development.
//! Rust-GB is a both toolchain and library for compiling Rust code into
//! Nintendo GameBoy. It compiles Rust code into valid GameBoy ROM through
//! LLVM-CBE and GBDK-2020.
//!
//! ## Install the compiler
//! Install a Rust-GB compiler with `cargo install`
//! 
//! ``` bash
//! cargo install rust-gb --features=compiler
//! ```
//!
//! `compiler` feature is required when installing the Rust-GB compiler.
//! If not, binary will not be installed.
//!
//! > **Note:** Right now, you can't run the Rust-GB compiler other than Linux x64. This is because the
//! > GameBoy compilation process requires some "external binaries". We're going to solve this problem in
//! > the future by packaging Rust-GB compiler for a platform-specific package manager (`winget`, `homebrew`, `pacman` etc.)
//!
//! ## Setup a project
//! Rust-GB ROM project have to provide all the right default settings so `cargo build` will just work.
//! Most recommended way to do this is cloning [`rust-gb-template`](https://github.com/zlfn/rust-gb-template) 
//! repository.
//! 
//! ```bash
//!  git clone https://github.com/zlfn/rust-gb-template.git
//!  ```
//! 
//! This repository contains minimum files to be compiled GameBoy ROM properly.
//! 
//! ## Compile your project
//! By executing the `cargo build-rom` command inside you GameBoy ROM project, you can compile the
//! Rust code into GameBoy ROM.
//!
//! The command creates two directories: `out` and `ext`.
//!
//! * **out** : In this directory, all intermediates generated during the compilation process
//! (LLVM-IR, C, ASM etc.) and the final result `out.gb` are generated.
//!
//! * **ext** : GameBoy ROM builds require external binaries (SDCC, LLVM-CBE) and dependency files.
//! By default, the Rust-GB compiler contains these files, but when compile GameBoy ROM, it needs
//! to be copied to the file system. This directory contains those external dependency files.
//!
//! ## Execute your ROM
//! The final result, `out.gb`, is located in the `out` directory. This file can be run using the
//! GameBoy emulator or real GameBoy (Color / Advance).
//!
//! The most recommended emulator is [bgb](https://bgb.bircd.org/). However, unless there is a
//! major problem, any GameBoy emulator should be able to run the `out.gb` file.
#![doc = document_features::document_features!()]
#![doc(html_logo_url = "https://github.com/zlfn/rust-gb/blob/main/media/ferris-gb.png?raw=true")]

#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]
#![feature(doc_cfg)]

pub mod io;

pub mod mmio;

pub mod drawing;

#[cfg(feature = "prototype")]
pub mod memory;

pub mod gbdk_c;

