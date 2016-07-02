// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
pub use self::common::*;

const INIT_OFFSET: isize = 1;

macro_rules! init {
  ($stack_ptr:expr, $closure_ptr:expr, $tramp_ptr:expr) => {
    // Initialise a new context
    //
    // Local passes:
    //  * esi: stack pointer
    //  * ebx: trampoline pointer
    //  * edx: closure pointer
    //
    // Context swapping to new context passes:
    //  * esi: their stack pointer (pass through)
    //  * edi: args pointer
    //
    // Local gets:
    //  * esi: new stack pointer
    asm!(
      r#"
      1:
        // Switch to the fresh stack
        xchg %esp, %esi

        // Save for trampoline call
        pushl %edx // closure
        pushl %ebx // trampoline
        // Save the return address, control flow continues at label 3
        call 3f
      2:
        // We arrive here once this context is reactivated (see swap.s)

        // Initialise the frame pointer
        movl $$0, %ebp

        // Restore trampoline pointer
        popl %ebx

        // Closure pointer stays on stack as third argument

        // Old stack pointer goes as second argument
        pushl %esi

        // Args for closure goes as first argument
        pushl %edi

        // Call the trampoline
        call *%ebx

        // crash if it ever returns
        ud2

      3:
        // Save our neatly-setup new stack
        xchg %esp, %esi

        // Back into Rust-land we go!
      "#
      : "={esi}" ($stack_ptr)
      : "{esi}"  ($stack_ptr),
        "{ebx}"  ($tramp_ptr),
        "{edx}"  ($closure_ptr)
      :
      : "volatile")
  };
}

macro_rules! swap {
  ($stack_ptr:expr, $params:expr, $args:expr) => {
    // Switch to a new context
    //
    // Local passes:
    //  * esi: new stack pointer
    //  * edi: pointer to args for new context
    //
    // Context swapping to here passes:
    //  * esi: theirforeign stack pointer (pass through)
    //  * edi: black-box args for this context
    //
    // Local gets:
    //  * esi: foreign stack pointer (pass through)
    //  * edi: black-box args for this context (pass through)
    //
    // Context swapped from here gets:
    //  * esi: local stack pointer
    //  * edi: black-box args for new context (pass through)
    asm!(
      r#"
      1:
        // Save the base pointer -- LLVM screws up when we make it responsible
        pushl %ebp

        // Save the return address to the stack
        pushl $$2f

        // Swap the stack pointers
        xchg %esp, %esi

        // Jump into the new context (return to the call point).
        //
        // Doing this instead of a straight `ret` is 8ns slower,
        // presumably because the branch predictor tries to be clever about it
        //
        // ebx is clobbered but that is OK
        popl %ebx
        jmp *%ebx

      2:
        // We arrive here once this context is reactivated

        // Restore the base pointer
        popl %ebp

        // And we merrily go on our way, back into Rust-land
      "#
      : "={esi}" ($stack_ptr),
        "={edi}" ($params)
      : "{esi}"  ($stack_ptr),
        "{edi}"  ($args)
      // LLVM doesn't back up the base pointer right at all...!
      : "eax",  "ebx",  "ecx",  "edx",  "esi",  "edi", //"ebp",  "esp",
        "mmx0", "mmx1", "mmx2", "mmx3", "mmx4", "mmx5", "mmx6", "mmx7",
        "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
        "cc", "fpsr", "eflags"
      : "volatile")
  };
}

#[path = "x86_common.rs"]
mod common;
