#![feature(unboxed_closures, default_type_params, asm)]
extern crate test;
extern crate libc;
extern crate lwkt;
use test::Bencher;
use lwkt::Context;
use std::ptr::null_mut;
use std::mem::{transmute, forget};

#[bench]
fn swap(b: &mut Bencher) {
  let mut native = unsafe { Context::native() };
  let mut green = Context::new(move |:| unsafe {
    loop { Context::swap(&mut *green, &mut *native); }
  });

  unsafe {
    Context::swap(&mut native, &mut green);
  }

  b.iter(|| unsafe {
    Context::swap(&mut native, &mut green);
  })
}

#[bench]
fn kernel_swap(b: &mut Bencher) {
  b.iter(|| unsafe {
    asm!("movq $$102, %rax\n\
          syscall"
         :
         :
         : "rax", "rcx"
         : "volatile");
  });
}
