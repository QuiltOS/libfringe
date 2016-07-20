// This file is part of libfringe, a low-level green threading library.
// Copyright (c) whitequark <whitequark@whitequark.org>
//               John Ericson <Ericson2314@Yahoo.com>
// See the LICENSE file included in this distribution.

//! Adaptor methods for types that are bigger than a CPU Word
use core::mem;
use core::ptr;

use void::{self, Void};

use stack_pointer::StackPointer;

/*
struct Fun<F>(F);

impl<A, F> FnOnce<(StackPointer, *mut A)> for Fun<F>
  where F: FnOnce<(StackPointer, A)>
{
type Output = <F as FnOnce<(StackPointer, A)>>::Output;

  extern "rust-call" fn call_once(
    self,
    (regs, args): (StackPointer, *mut A))
    -> Self::Output
  {
    self.0.call_once((regs, unsafe { ptr::read(args) }))
  }
}
*/

const NUM_REGS: usize = 2;

#[inline(always)]
pub unsafe fn from_regs<F>(regs: [usize; NUM_REGS]) -> F {
  ptr::read(if mem::size_of::<F>() <= mem::size_of::<F>() * NUM_REGS {
    // in regs
    &regs as *const _ as *const _
  } else {
    // via pointer
    regs[0] as *const _
  })
}

#[inline(always)]
pub unsafe fn to_regs<F>(closure_ptr: *const F) -> [usize; NUM_REGS] {
  let mut regs: [usize; NUM_REGS] = [mem::uninitialized(), mem::uninitialized()];
  if mem::size_of::<F>() <= (mem::size_of::<F>() * NUM_REGS) {
    // in regs
    ptr::write(&mut regs as *mut _ as *mut _,
               ptr::read(closure_ptr));
  } else {
    // via pointer
    regs[0] = closure_ptr as usize;
  }
  regs
}


// Adapted from whitquark's generator

/// Initializes a stack with a closure *and switches*.
///
/// It is the responsibility of the closure to immediately yeild `R`
/// if control wishes to be returned to caller immediately. Use a
/// reference if closure is a DST.
pub unsafe fn init<F, R>(sp: &mut StackPointer, closure: F) -> R
  where F: FnOnce(StackPointer) -> Void
{
  unsafe extern "C" fn closure_wrapper<F>(
    sp: StackPointer, a0: usize, a1: usize) -> !
    where F: FnOnce(StackPointer) -> Void
  {
    let closure: F = from_regs::<F>([a0, a1]);
    void::unreachable(closure(sp))
  }

  sp.init(closure_wrapper::<F>);
  let (sp2, ret) = swap(ptr::read(sp), closure);
  ptr::write(sp, sp2);
  ret
}

/*

/// `A` can be any size
#[inline]
pub unsafe fn fat_new<'a, A, F>(top:     *mut u8,
                                closure: F)
                                -> (StackPointer, *mut F)
  where F: FnOnce(StackPointer, A) -> Void + 'a,
{
  let (regs, ptr) = StackPointer::init::<'a, *mut A, _>(top, Fun(closure));
  (regs, &mut (*ptr).0 as *mut _)
}
 */

/// `I` and `O` can be any size
pub unsafe fn swap<I, O>(new_sp: StackPointer, args: I) -> (StackPointer, O) {
  let [arg0, arg1] = to_regs(&args);
  let (old_sp, param0, param1) = StackPointer::swap(new_sp, arg0, arg1);
  mem::forget(args);
  (old_sp, from_regs([param0, param1]))
}
