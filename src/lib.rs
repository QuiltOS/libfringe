#![feature(default_type_params, phase, globs, asm)]
#![no_std]

#[phase(plugin, link)]
extern crate core;
extern crate alloc;

pub use context::Context;

mod std { pub use core::fmt; }

mod context;
mod stack;

mod platform;
