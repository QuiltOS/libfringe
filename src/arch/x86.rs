// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
pub use self::common::*;

macro_rules! init {
  ($sp:expr, $f_ptr:expr, $tramp:expr) => {
    // initialise a new context
    // arguments:
    //  * eax: stack pointer
    //  * ebx: function pointer
    //  * ecx: data pointer
    //
    // return values:
    //  * eax: new stack pointer
    asm!(
      r#"
        // switch to the fresh stack
        xchg %esp, %eax

        // save the data pointer and the function pointer, respectively
        pushl %ecx
        pushl %ebx

        // save the return address, control flow continues at label 1
        call 1f
        // we arrive here once this context is reactivated (see swap.s)

        // restore the function pointer (the data pointer is the first argument, which lives at the top of the stack)
        popl %eax

        // initialise the frame pointer
        movl $$0, %ebp

        // call the function pointer with the data pointer (top of the stack is the first argument)
        call *%eax

        // crash if it ever returns
        ud2

      1:
        // save our neatly-setup new stack
        xchg %esp, %eax
        // back into Rust-land we go
      "#
      : "={eax}"($sp)
      : "{eax}" ($sp),
        "{ebx}" ($tramp),
        "{ecx}" ($f_ptr)
      :
      : "volatile")
  };
}

macro_rules! swap {
  ($out_spp:expr, $in_spp:expr) => {
    // switch to a new context
    // arguments:
    //  * eax: stack pointer out pointer
    //  * ebx: stack pointer in pointer
    asm!(
      r#"
        // save the frame pointer
        pushl %ebp

        // save the return address to the stack, control flow continues at label 1
        call 1f
        // we arrive here once this context is reactivated

        // restore the frame pointer
        popl %ebp

        // and we merrily go on our way, back into Rust-land
        jmp 2f

      1:
        // retrieve the new stack pointer
        movl (%eax), %edx
        // save the old stack pointer
        movl %esp, (%ebx)
        // switch to the new stack pointer
        movl %edx, %esp

        // jump into the new context (return to the call point)
        // doing this instead of a straight `ret` is 8ns slower,
        // presumably because the branch predictor tries to be clever about it
        popl %eax
        jmpl *%eax

      2:
      "#
      :
      : "{eax}" ($out_spp),
        "{ebx}" ($in_spp)
      : "eax",  "ebx",  "ecx",  "edx",  "esi",  "edi", //"ebp",  "esp",
        "mmx0", "mmx1", "mmx2", "mmx3", "mmx4", "mmx5", "mmx6", "mmx7",
        "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
        "cc", "fpsr", "eflags"
      : "volatile")
  };
}

#[path = "x86_common.rs"]
mod common;
