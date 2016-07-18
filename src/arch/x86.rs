// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>,
//               whitequark <whitequark@whitequark.org>
//               John Ericson <Ericson2314@Yahoo.com>
// See the LICENSE file included in this distribution.

//! To understand the code in this file, keep in mind this fact:
//! * i686 SysV C ABI requires the stack to be aligned at function entry,
//!   so that `%esp+4` is a multiple of 16. Aligned operands are a requirement
//!   of SIMD instructions, and making this the responsibility of the caller
//!   avoids having to maintain a frame pointer, which is necessary when
//!   a function has to realign the stack from an unknown state.
//! * i686 SysV C ABI passes the first argument on the stack. This is
//!   unfortunate, because unlike every other architecture we can't reuse
//!   `swap` for the initial call, and so we use a trampoline.
use stack_pointer::StackPointer;

#[inline(always)]
pub unsafe fn init(
  mut sp: StackPointer,
  fun: unsafe extern "C" fn(StackPointer, usize, usize) -> !)
  -> StackPointer
{
  #[naked]
  unsafe extern "C" fn trampoline() -> ! {
    asm!(
      r#"
        # Pop function.
        popl    %ebx
        # Push arguments.
        pushl   %edx
        pushl   %esi
        pushl   %eax
        # Call it.
        call    *%ebx
      "# ::: "memory" : "volatile");
    ::core::intrinsics::unreachable()
  }

  sp.push(0usize); // alignment
  sp.push(fun as usize);
  sp.push(trampoline as usize);
  sp
}

#[inline(always)]
pub unsafe fn swap(mut new_sp: StackPointer,
                   mut arg0: usize,
                   mut arg1: usize)
                   -> (StackPointer, usize, usize)
{
  asm!(
    r#"
      # Save frame pointer explicitly; LLVM doesn't spill it even if it is
      # marked as clobbered.
      pushl   %ebp
      # Push instruction pointer of the old context and switch to
      # the new context.
      call    1f
      # Restore frame pointer.
      popl    %ebp
      # Continue executing old context.
      jmp     2f

    1:
      # Swap current stack pointer with the new stack pointer
      xchg    %esp, %eax

      # Pop instruction pointer of the new context (placed onto stack by
      # the call above) and jump there; don't use `ret` to avoid return
      # address mispredictions (~8ns on Ivy Bridge).
      popl    %ebx
      jmpl    *%ebx
    2:
    "#
    : "={eax}" (new_sp.0)
      "={esi}" (arg0)
      "={edx}" (arg1)
    : "{eax}" (new_sp.0)
      "{esi}" (arg0)
      "{edx}" (arg1)
    : "eax",  "ebx",  "ecx",  "edx",  "esi",  "edi", //"ebp",  "esp",
      "mmx0", "mmx1", "mmx2", "mmx3", "mmx4", "mmx5", "mmx6", "mmx7",
      "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
      "cc", "fpsr", "flags", "memory"
    : "volatile");
  (new_sp, arg0, arg1)
}
