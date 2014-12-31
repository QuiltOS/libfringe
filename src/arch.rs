use core::prelude::*;
use core::simd::u64x2;
use core::mem::{
  size_of,
  zeroed,
  swap,
  transmute,
};

use stack::Stack;
use context::Context;

/// stack must be new/usused to preserve memory safety of objects on
/// stack
#[inline(always)]
pub fn initialise_call_frame(stack:     &mut Stack,
                             init:      fn() -> ())
{
  let sp    = stack.top();
  let limit = stack.limit();
  unsafe {
    asm!("pushq $0" :: "r" (init)  :: "volatile"); // for bootstrap
    asm!("pushq $0" :: "r" (stack) :: "volatile"); // for bootstrap
    asm!("pushq boostrap"         :::: "volatile"); // for rip
    asm!("pushq"
         :
         : "{rdi}" (sp) "{0}" (limit)
         : "rax", "rbx", "rcx", "rdx", "rbp", /* "rsp",*/ "rsi", "rdi",
         "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
         "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
         "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15"
         : "volatile");
    asm!("popq %rdi
        popq %rdi
        popq %rdi"
       :
       :
       : "rdi"
       : "volatile"); // compensate for earlier pops
  }
}

#[allow(unused)]
unsafe fn __bootstrap() {
  panic!();

  asm!("bootstrap:" :::: "volatile"); // enter here
  {
    let f:     fn() -> ();
    let stack: *mut Stack;
    asm!("popq $0
          popq $1"
         : "=r" (stack), "=r" (f)
         :
         :
         : "volatile");
    f();
    let mut temp: Stack = zeroed();
    swap(&mut temp, transmute(stack));
  }
  panic!();
}


#[inline(always)]
pub unsafe fn swap_stack(stack_ptr: uint) {
  asm!("pushq %rip" :::: "volatile");
  asm!("pushq %fs:0x70
        pushq %rip
        xchg %rsp, $0
        popq %rip
        popq %fs:0x70"
       :
       : "{+rdi}" (stack_ptr) //+rdi to mimmick above
       : "rax", "rbx", "rcx", "rdx", "rbp", /* "rsp",*/ "rsi", "rdi",
       "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
       "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
       "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15"
       : "volatile");
  asm!("popq %rip" :::: "volatile");
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
