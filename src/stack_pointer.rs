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
    fun: unsafe extern "C" fn(usize) -> !)
    -> StackPointer
  {
    let mut sp = StackPointer(stack.base() as _);
    ::arch::init(&mut sp, fun);
    sp
  }

  #[inline(always)]
  pub unsafe fn swap(arg: usize,
                     old_sp: &mut StackPointer,
                     new_sp: &StackPointer,
                     new_stack: &Stack) -> usize{
    ::arch::swap(arg, old_sp, new_sp, new_stack)
  }
}

pub unsafe fn align_down_mut<T>(sp: *mut T, n: usize) -> *mut T {
  let sp = (sp as usize) & !(n - 1);
  sp as *mut T
}
