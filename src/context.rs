use core::prelude::*;

use core::intrinsics::abort;
use core::ptr;

use stack::Stack;
use arch::{mod, Registers};

pub struct Context {
  regs: Registers,
  stack: Stack
}

impl Context {
  pub fn new<F: FnOnce()>(f: F) -> Context {
    let mut stack = Stack::new(4 << 20);

    unsafe {
      let mut ctx = Context {
        regs: Registers::new(),
        stack: Stack::native(-1 as *const u8)
      };

      let mut my_ctx = Context::native();

      ctx.regs = arch::initialise_call_frame(&mut stack,
        init_ctx::<F> as arch::uintptr_t,
        &[&f as *const F as arch::uintptr_t,
          &mut ctx as *mut Context as arch::uintptr_t,
          &mut my_ctx as *mut Context as arch::uintptr_t]);

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
      regs: Registers::new(),
      stack: Stack::native(arch::get_sp_limit())
    }
  }


  #[inline(always)]
  pub unsafe fn swap(out_context: &mut Context, in_context: &mut Context) {
    arch::set_sp_limit(in_context.stack.limit());
    arch::swapcontext(&mut out_context.regs, &mut in_context.regs);
  }
}
