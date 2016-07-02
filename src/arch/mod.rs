// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
pub use self::imp::*;

unsafe impl Send for Registers {}

mod common;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod imp;

#[cfg(target_arch = "x86")]
#[path = "x86.rs"]
mod imp;
