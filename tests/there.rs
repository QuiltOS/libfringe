// This file is part of libfringe, a low-level green threading library.
// Copyright (c) 2015, Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
extern crate env_logger;
extern crate fringe;
use fringe::NATIVE_THREAD_LOCALS;
use fringe::pattern::cycle::{C1, Cycle};

use std::process;

#[test]
fn main() {
  env_logger::init().unwrap();
  let stack = fringe::OsStack::new(4 << 20).unwrap();

  let ctx = C1::new(stack, move |_, ctx| {
    assert!(ctx.0.is_none());
    println!("it's alive!");
    process::exit(0)
  });

  ctx.kontinue(NATIVE_THREAD_LOCALS, ());
}
