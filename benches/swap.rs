// This file is part of libfringe, a low-level green threading library.
// Copyright (c) 2015, Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
#![feature(test)]
#![cfg(feature = "os")]
extern crate test;
extern crate fringe;
use fringe::NATIVE_THREAD_LOCALS;
use fringe::pattern::cycle::{C1, Cycle};

#[bench]
fn swap(b: &mut test::Bencher) {
  unsafe {
    let stack = fringe::OsStack::new(4 << 20).unwrap();

    let mut ctx: C1<'static, ()> = C1::new(stack, move |tl, (mut ctx, ())| loop {
      ctx = ctx.unwrap().swap(Some(tl), ()).0;
    });

    ctx = ctx.swap(NATIVE_THREAD_LOCALS, ()).0.unwrap();

    b.iter(move || {
      use std::mem::{swap, uninitialized, forget};

      let mut tmp = uninitialized();
      swap(&mut ctx, &mut tmp);

      tmp = tmp.swap(NATIVE_THREAD_LOCALS, ()).0.unwrap();

      swap(&mut ctx, &mut tmp);
      forget(tmp);
    });
  }
}
