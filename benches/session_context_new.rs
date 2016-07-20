// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
#![feature(test)]
extern crate test;
extern crate fringe;
use fringe::Stack;
use fringe::NATIVE_THREAD_LOCALS;
use fringe::pattern::cycle::{C1, Cycle};


static mut stack_buf: [u8; 1024] = [0; 1024];

#[bench]
fn context_new(b: &mut test::Bencher) {
  b.iter(|| unsafe {
    let stack = SliceStack(&mut stack_buf);

    let ctx: C1<'static, ()> = C1::new(stack, move |tl, (ctx, ())| {
      ctx.unwrap().kontinue(Some(tl), ())
    });

    ctx.swap(NATIVE_THREAD_LOCALS, ());
  })
}

#[bench]
fn context_new_with_dead_loop(b: &mut test::Bencher) {
  b.iter(|| unsafe {
    let stack = SliceStack(&mut stack_buf);

    let ctx: C1<'static, ()> = C1::new(stack, move |tl, (mut ctx, ())| loop {
      ctx = ctx.unwrap().swap(Some(tl), ()).0;
    });

    ctx.swap(NATIVE_THREAD_LOCALS, ());
  })
}

struct SliceStack<'a>(&'a mut [u8]);
impl<'a> fringe::Stack for SliceStack<'a> {
  fn top(&mut self) -> *mut u8 {
    unsafe {
      self.0.as_mut_ptr().offset(self.0.len() as isize)
    }
  }

  fn limit(&self) -> *const u8 {
    self.0.as_ptr()
  }
}
