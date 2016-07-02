// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
pub use self::common::*;

const INIT_OFFSET: isize = 0;

macro_rules! init {
  ($stack_ptr:expr, $closure_ptr:expr, $tramp_ptr:expr) => {
    // Initialise a new context
    //
    // Local passes:
    //  * rsi: stack pointer
    //  * rbx: trampoline pointer
    //  * rdx: closure pointer
    //
    // Context swapping to new context passes:
    //  * rsi: their stack pointer (pass through)
    //  * rdi: args pointer
    //
    // Local gets:
    //  * eax: new stack pointer
    asm!(
      r#"
      1:
        // Switch to the fresh stack
        xchg %rsp, %rsi

        // Save for trampoline call
        pushq %rdx // closure
        pushq %rbx // trampoline

        // Save the return address, control flow continues at label 3
        call 3f
      2:
        // We arrive here once this context is reactivated (see swap.s)

        // Initialise the frame pointer
        movq $$0, %rbp

        // Restore trampoline pointer
        popq %rbx

        // Closure pointer popped off stack into reg for third argument
        popq %rdx

        // Old stack pointer stays in reg as second argument

        // Args for closure stays in reg as first argument

        // Call the trampoline
        call *%rbx

        // crash if it ever returns
        ud2

      3:
        // Save our neatly-setup new stack
        xchg %rsp, %rsi

        // Back into Rust-land we go!
      "#
      : "={rsi}" ($stack_ptr)
      : "{rsi}"  ($stack_ptr),
        "{rbx}"  ($tramp_ptr),
        "{rdx}"  ($closure_ptr)
      :
      : "volatile")
  };
}

macro_rules! swap {
  ($stack_ptr:expr, $params:expr, $args:expr) => {
    // Switch to a new context
    //
    // Local passes:
    //  * rsi: new stack pointer
    //  * rdi: pointer to args for new context
    //
    // Context swapping to here passes:
    //  * rsi: theirforeign stack pointer (pass through)
    //  * rdi: black-box args for this context
    //
    // Local gets:
    //  * rsi: foreign stack pointer (pass through)
    //  * rdi: black-box args for this context (pass through)
    //
    // Context swapped from here gets:
    //  * rsi: local stack pointer
    //  * rdi: black-box args for new context (pass through)
    asm!(
      r#"
      1:
        // Make sure we leave the red zone alone
        sub $$128, %rsp

        // Save the base pointer -- LLVM screws up when we make it responsible
        pushq %rbp

        // Save the return address to the stack
        leaq 2f(%rip), %rax
        pushq %rax

        // Swap the stack pointers
        xchg %rsp, %rsi

        // Jump into the new context (return to the call point).
        //
        // Doing this instead of a straight `ret` is 8ns slower,
        // presumably because the branch predictor tries to be clever about it
        //
        // ebx is clobbered but that is OK
        popq %rbx
        jmp *%rbx

      2:
        // We arrive here once this context is reactivated

        // Restore the base pointer
        popq %rbp

        // Give back the red zone
        add $$128, %rsp

        // And we merrily go on our way, back into Rust-land
      "#
      : "={rsi}" ($stack_ptr),
        "={rdi}" ($params)
      : "{rsi}"  ($stack_ptr),
        "{rdi}"  ($args)
      // LLVM doesn't back up the base pointer right at all...!
      : "rax",   "rbx",   "rcx",   "rdx",   "rsi",   "rdi", //"rbp",   "rsp",
        "r8",    "r9",    "r10",   "r11",   "r12",   "r13",   "r14",   "r15",
        "xmm0",  "xmm1",  "xmm2",  "xmm3",  "xmm4",  "xmm5",  "xmm6",  "xmm7",
        "xmm8",  "xmm9",  "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15",
        "xmm16", "xmm17", "xmm18", "xmm19", "xmm20", "xmm21", "xmm22", "xmm23",
        "xmm24", "xmm25", "xmm26", "xmm27", "xmm28", "xmm29", "xmm30", "xmm31"
        "cc", "fpsr", "eflags"
      : "volatile")
  };
}

#[path = "x86_common.rs"]
mod common;
