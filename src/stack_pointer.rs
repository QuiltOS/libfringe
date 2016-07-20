use core::mem::align_of;
use core::ptr;

use stack::Stack;

#[derive(Clone, Debug)]
/// The bare-minimum context, largely unsafe to use but exposed for the building
/// of other abstractions.
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

  #[inline]
  pub fn new<S: Stack>(stack: &S) -> StackPointer {
    StackPointer(stack.top() as *mut _)
  }

  pub unsafe fn init(
    &self,
    fun: unsafe extern "C" fn(StackPointer, usize, usize) -> !)
    -> StackPointer
  {
    ::arch::init(StackPointer(self.0), fun)
  }

  #[inline(always)]
  pub unsafe fn swap(new_sp: StackPointer,
                     arg0: usize,
                     arg1: usize)
                     -> (StackPointer, usize, usize)
  {
    ::arch::swap(new_sp, arg0, arg1)
  }
}

pub unsafe fn align_down_mut<T>(sp: *mut T, n: usize) -> *mut T {
  let sp = (sp as usize) & !(n - 1);
  sp as *mut T
}
