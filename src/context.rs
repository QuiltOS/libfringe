use core::prelude::*;

use core::kinds::marker::ContravariantType;

use stack::Stack;

#[path = "arch.rs"]
mod arch;


pub struct Context<T> {
  _stack:     Stack, 
  stack_ptr: *const u8,
  _marker:   ContravariantType<T>,
}

impl Context<()>
{
  pub fn new<F: FnOnce() + 'static>(f: F) -> Context<()> {
    let mut stack = Stack::new(4 << 20);
    let sp    = stack.top();

    let mut them = Context {
      _stack:     stack,
      stack_ptr: sp,
      _marker:   ContravariantType,
    };

    let slot: &mut F = unsafe {
      let mut us: Context<&mut F> =
        ::core::mem::transmute(Context::native());

      arch::bootstrap(&mut them, &mut us); // backwards because from opposite perspective
      swap((), &mut us, &mut them)
    };

    // move closure to new stack
    *slot = f;

    them
  }

  pub unsafe fn native() -> Context<()> {
    Context {
      _stack:     Stack::native(arch::get_sp_limit()),
      stack_ptr: arch::get_sp() as *mut u8,
      _marker:   ContravariantType,
    }
  }
}

#[inline(always)]
pub fn swap<'t, A, B>(args:        A,
                      out_context: &'t mut Context<B>,
                      in_context:  &'t mut Context<A>) -> B
{
  unsafe {
    out_context.stack_ptr = in_context.stack_ptr;
    let f: unsafe extern "fastcall" /*"anyreg"*/ fn(A, &mut *const u8) -> B
      = ::core::mem::transmute(arch::swap::<A>);

    f(args, &mut out_context.stack_ptr)
  }
}
