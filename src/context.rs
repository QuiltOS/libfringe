use core::prelude::*;

use core::intrinsics::abort;
use core::ptr;

use stack::Stack;
use arch;

pub struct Context<T> {
  stack:     Stack,
  stack_ptr: uint,
  _marker:   ::core::kinds::marker::ContravariantType,
}

impl Context {
  pub fn new<F: FnOnce()>(f: F) -> Context {
    let mut stack = Stack::new(4 << 20);

    unsafe {
      let mut ctx = Context {
        stack: Stack::native(-1 as *const u8)
      };

      let mut my_ctx = Context::native();

      arch::initialise_call_frame(&mut stack,
        init_ctx::<F> as uint,
        &[&f          as *const F as uint,
          &mut ctx    as *mut Context as uint,
          &mut my_ctx as *mut Context as uint]);

      ctx.stack = stack;

      Context::swap(&mut my_ctx, &mut ctx);

      ctx
    }
  }
}

unsafe extern "C" fn init_ctx<F: FnOnce()>(f: *const F, ctx: *mut Context,
                                           parent_ctx: *mut Context) -> ! {
  let f = ptr::read(f);
  Context::swap(&mut *ctx, &mut *parent_ctx);
  f();
  abort();
}

impl Context {
  pub unsafe fn native() -> Context {
    Context {
      stack: Stack::native(arch::get_sp_limit())
    }
  }
}

#[inline(always)]
pub fn swap<A, B>(args:        A,
                  out_context: &mut Context<B>,
                  in_context:  &mut Context<A>) -> B
{
  unsafe {
    out_context.stack_ptr = in_context.stack_ptr;
    let f: unsafe extern "fastcall" /*"anyreg"*/ fn(A, *mut u8) -> B
      = transmute(swap_help::<A>);
    
    f(args, &mut out_context.stack_ptr)
  }
}
