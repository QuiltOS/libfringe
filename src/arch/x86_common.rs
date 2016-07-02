// This file is part of libfringe, a low-level green threading library.
// Copyright (c) 2015, Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
use void::Void;

use arch::common::{push, rust_trampoline, align_down_mut};
use super::INIT_OFFSET;

pub const STACK_ALIGN: usize = 16;

#[derive(Debug, Copy, Clone)]
pub struct Registers {
  stack_pointer: *mut usize,
}

impl Registers {
  /// `A` must be the same size as a CPU Word
  #[inline]
  pub unsafe fn new<'a, A, F>(top:     *mut u8,
                              closure: F)
                              -> (Self, *mut F)
    where F: FnOnce(Registers, A) -> Void + 'a,
  {
    let mut stack_ptr   = top as *mut usize;
    let     closure_ptr = push(&mut stack_ptr, closure);


    // align stack to make sure trampoline is called properly
    stack_ptr = stack_ptr.offset(INIT_OFFSET);
    stack_ptr = align_down_mut(stack_ptr, STACK_ALIGN);
    stack_ptr = stack_ptr.offset(-INIT_OFFSET);

    init!(stack_ptr,
          closure_ptr,
          rust_trampoline::<A, F> as unsafe extern "C" fn(A, _, _) -> !);

    (Registers { stack_pointer: stack_ptr }, closure_ptr)
  }

  /// `I` and `O` must be the same size as a CPU Word
  #[inline(always)]
  pub unsafe fn swap<I, O>(mut self, args: I) -> (Self, O) {
    let params: O;
    swap!(self.stack_pointer, params, args);
    (self, params)
  }
}
