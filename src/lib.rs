// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
#![feature(asm, fn_traits, unboxed_closures, unique, unsize)]
#![no_std]

//! libfringe is a low-level green threading library.
//! It provides only a context-swapping mechanism.

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate log;

extern crate void;

pub use context::{Context, ThreadLocals, NATIVE_THREAD_LOCALS};
pub use stack::Stack;

#[cfg(feature = "os")]
pub use os::Stack as OsStack;

pub mod pattern;

mod context;
mod stack;

#[cfg(feature = "os")]
mod os;

mod arch;
mod fat_args;
mod debug;

#[cfg(not(test))]
mod std { pub use core::*; }
