// This file is part of libfringe, a low-level green threading library.
// Copyright (c) 2015, Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
extern crate env_logger;
extern crate fringe;

use std::vec::Vec;

use fringe::NATIVE_THREAD_LOCALS;
use fringe::pattern::cycle::{C1, Cycle};

#[test]
fn main() {
  env_logger::init().unwrap();
  let stack = fringe::OsStack::new(4 << 20).unwrap();

  let mut vec: Vec<&'static str> = vec!("The begining entry");

  let mut ctx: C1<'static, _> = C1::new(stack, |tl, (mut ctx, mut vec): (Option<C1<'static, _>>, Vec<&'static str>)| {
    loop {
      println!("so far: {:?}", vec);
      match vec.len() {
        x if x < 5 => vec.push("Here is another entry"),
        x if x < 6 => vec.push("Here is the final entry"),
        _          => ctx.unwrap().kontinue(Some(tl), vec),
      };
      let x = ctx.unwrap().swap(Some(tl), vec);
      ctx = x.0;
      vec = x.1;
    }
  });

  loop {
    let (c, v) = ctx.swap(NATIVE_THREAD_LOCALS, vec);
    vec = v;
    match c {
      None    => break,
      Some(c) => ctx = c,
    }
  }

  assert_eq!(vec.len(), 6);
}
