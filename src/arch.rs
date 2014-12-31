use core::prelude::*;

use context::{mod, Context};

// TODO use anyreg and feel better about nothing on stack
// TODO find some way to ensure no rust temporaries or arguments on the stack
#[allow(unused)]
pub unsafe extern "fastcall" fn bootstrap
  <F: FnOnce() + 'static>(us: &mut Context<()>, spawner: &mut Context<&'static mut F>)
{
  // jank switch stacks / simulate cdecl
  asm!("xchg $0 %rsp
        push bootstrap_propper"
       :
       : "r" (us.stack_ptr)
       : // what clobber ;)
       : "volatile");

  asm!("pushq %fs:0x70
        mov ($0), %rsi
        jump skip: # jk lol
        # THE REST OF THIS ASM BLOCK IS DEAD
        xchg %rsp, %rsi
        mov %rsi, ($0)
        popq %fs:0x70"
       :
       : "{rdi}" (spawner.stack_ptr)
       : "rax", "rbx", "rcx", "rdx", "rbp", /* "rsp",*/ "rsi", //"rdi",
       "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
       "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
       "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15"
       : "volatile");

  asm!("xchg $0 %rsp
        ret;"
       :
       : "r" (us.stack_ptr)
       : // what clobber ;)
       : "volatile");

  asm!("bootstrap_proper:" :::: "volatile");
  {
    let mut closure: F = ::core::mem::uninitialized();
    // init closure
    // TODO forall lifetime . type
    context::swap(::core::mem::transmute(&mut closure), us, spawner);
    closure();
  }
  panic!("now what?!");
}

// TODO use anyreg and no longer need clobbers
#[inline(never)]
pub unsafe extern "fastcall" fn swap
  <A>(args: A, stack_ptr: &mut *mut u8) -> A
{
  asm!("pushq %fs:0x70
        mov ($0), %rsi
        xchg %rsp, %rsi
        mov %rsi, ($0)
        popq %fs:0x70"
       :
       : "{rdi}" (stack_ptr)
       : "rax", "rbx", "rcx", "rdx", "rbp", /* "rsp",*/ "rsi", //"rdi",
       "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
       "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
       "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15"
       : "volatile");
  args
}


// Rust stores a stack limit at [fs:0x70]. These two functions set and retrieve
// the limit. They're marked as #[inline(always)] so that they can be used in
// situations where the stack limit is invalid.

#[inline(always)]
pub unsafe fn get_sp() -> *const u8 {
  let sp;
  asm!("mov %rsp, $0" : "=r"(sp) ::: "volatile");
  sp
}


#[inline(always)]
pub unsafe fn get_sp_limit() -> *const u8 {
  let limit;
  asm!("mov %fs:0x70, $0" : "=r"(limit) ::: "volatile");
  limit
}
/*
#[inline(always)]
pub unsafe fn set_sp(sp: *const u8) {
  asm!("mov $0, %rsp" :: "r"(sp) :: "volatile");
}

#[inline(always)]
pub unsafe fn set_sp_limit(limit: *const u8) {
  asm!("mov $0, %fs:0x70" :: "r"(limit) :: "volatile");
}

#[inline]
fn align_down_mut<T>(sp: *mut T, n: uint) -> *mut T {
  let sp = (sp as uint) & !(n - 1);
  sp as *mut T
}

// ptr::offset_mut is positive ints only
#[inline]
pub fn offset_mut<T>(ptr: *mut T, count: int) -> *mut T {
  (ptr as int + count * (size_of::<T>() as int)) as *mut T
}
*/
