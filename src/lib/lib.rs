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
//! Also, Rust-GB compiler requires complex toolchains, such as `llvm-cbe` or `lcc`.
//! Below command will install that kind of toolchains in cargo path.
//!
//! ```bash
//! cargo gb setup
//! ```
//!
//! ## Setup a project
//! 
//! ## Compile your project
//! ## Execute your ROM
#![doc = document_features::document_features!()]

#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]
#![feature(doc_cfg)]

pub mod io;

pub mod mmio;

#[cfg(feature = "with-gbdk")]
pub mod drawing;

#[cfg(feature = "prototype")]
pub mod memory;

pub mod gbdk_c;
