// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>,
//               whitequark <whitequark@whitequark.org>
//               John Ericson <Ericson2314@Yahoo.com>
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
use core::mem::align_of;
use core::ptr;

use stack::Stack;

#[derive(Debug, Clone)]
/// The bare-minimum context. It's quite unsafe
pub struct StackPointer(pub *mut usize);

impl StackPointer {
  pub unsafe fn push<T>(&mut self, value: T) -> *mut T {
    let mut sp = self.0 as *mut T;
    sp = sp.offset(-1);
    sp = align_down_mut(sp, align_of::<T>());
    ptr::write(sp, value); // does not attempt to drop old value
    self.0 = sp as *mut usize;
    sp
  }

  pub unsafe fn init(
    stack: &Stack,
    fun: unsafe extern "C" fn(StackPointer, usize, usize) -> !)
    -> StackPointer
  {
    let mut sp = StackPointer(stack.base() as _);
    ::arch::init(&mut sp, fun);
    sp
  }

  #[inline(always)]
  pub unsafe fn swap(new_stack: Option<&mut Stack>,
                     new_sp: StackPointer,
                     arg0: usize,
                     arg1: usize)
                     -> (StackPointer, usize, usize)
  {
    let cfa_slot = new_stack.map(|stack| {
      // Address of the topmost CFA stack slot.
      &mut *(stack.base() as *mut usize).offset(-1)
    });
    ::arch::swap(cfa_slot, new_sp, arg0, arg1)
  }
}

pub unsafe fn align_down_mut<T>(sp: *mut T, n: usize) -> *mut T {
  let sp = (sp as usize) & !(n - 1);
  sp as *mut T
}
