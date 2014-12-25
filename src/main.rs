#![feature(unboxed_closures, default_type_params, asm)]
extern crate lwkt;

use std::ptr::null_mut;
use std::intrinsics::abort;
use lwkt::Context;

fn main() {
  let mut native = unsafe { Context::native() };
  let mut green = Context::new(move |:| unsafe {
    println!("Hello, world!");
    Context::swap(&mut green, &mut native);
  });

  unsafe {
    Context::swap(&mut native, &mut green);
  }

  println!("size_of::<Context>() == {}", std::mem::size_of::<Context>());
}
