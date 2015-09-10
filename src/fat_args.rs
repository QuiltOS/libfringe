//! Adaptor methods for types that are bigger than a CPU Word
use core::ptr;

use void::Void;

use arch::Registers;

struct Fun<F>(F);

impl<A, F> FnOnce<(Registers, *mut A)> for Fun<F>
  where F: FnOnce<(Registers, A)>
{
  type Output = <F as FnOnce<(Registers, A)>>::Output;

  extern "rust-call" fn call_once(self, (regs, args): (Registers, *mut A)) -> Self::Output {
    self.0.call_once((regs, unsafe { ptr::read(args) }))
  }
}

impl Registers {
  /// `A` can be any size
  #[inline]
  pub unsafe fn fat_new<'a, A, F>(top:     *mut u8,
                                  closure: F)
                                  -> (Self, *mut F)
    where F: FnOnce(Registers, A) -> Void + 'a,
  {
    let (regs, ptr) = Registers::new::<'a, *mut A, _>(top, Fun(closure));
    (regs, &mut (*ptr).0 as *mut _)
  }

  /// `I` and `O` can be any size
  pub unsafe fn fat_swap<I, O>(self, mut args: I) -> (Self, O) {
    let (regs, params) = self.swap::<*mut I, *mut O>(&mut args as *mut _);
    ::core::mem::forget(args);
    (regs, ptr::read(params))
  }
}
