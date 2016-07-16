// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
pub use self::common::*;

macro_rules! init {
  ($sp:expr, $f_ptr:expr, $tramp:expr) => {
    // initialise a new context
    // arguments:
    //  * rdi: stack pointer
    //  * rsi: function pointer
    //  * rdx: data pointer
    //
    // return values:
    //  * rdi: new stack pointer
    asm!(
      r#"
        // switch to the fresh stack
        xchg %rsp, %rdi

        // save the function pointer the data pointer, respectively
        pushq %rsi
        pushq %rdx

        // save the return address, control flow continues at label 1
        call 1f
        // we arrive here once this context is reactivated (see swap.s)

        // restore the data pointer and the function pointer, respectively
        popq %rdi
        popq %rax

        // initialise the frame pointer
        movq $$0, %rbp

        // call the function pointer with the data pointer (rdi is the first argument)
        call *%rax

        // crash if it ever returns
        ud2

      1:
        // save our neatly-setup new stack
        xchg %rsp, %rdi
        // back into Rust-land we go
      "#
      : "={rdi}"($sp)
      : "{rdi}" ($sp),
        "{rsi}" ($tramp),
        "{rdx}" ($f_ptr)
      :
      : "volatile");
  }
}

macro_rules! swap {
  ($out_spp:expr, $in_spp:expr) => {
    // switch to a new context
    // arguments:
    //  * rdi: stack pointer out pointer
    //  * rsi: stack pointer in pointer
    asm!(
      r#"
        // make sure we leave the red zone alone
        sub $$128, %rsp

        // save the frame pointer
        pushq %rbp

        // save the return address to the stack, control flow continues at label 1
        call 1f
        // we arrive here once this context is reactivated

        // restore the frame pointer
        popq %rbp

        // give back the red zone
        add $$128, %rsp

        // and we merrily go on our way, back into Rust-land
        jmp 2f

      1:
        // retrieve the new stack pointer
        movq (%rsi), %rax
        // save the old stack pointer
        movq %rsp, (%rdi)
        // switch to the new stack pointer
        movq %rax, %rsp

        // jump into the new context (return to the call point)
        // doing this instead of a straight `ret` is 8ns faster,
        // presumably because the branch predictor tries
        // to be clever about it otherwise
        popq %rax
        jmpq *%rax

      2:
      "#
      :
      : "{rdi}" ($out_spp)
        "{rsi}" ($in_spp)
      : "rax",   "rbx",   "rcx",   "rdx",   "rsi",   "rdi", //"rbp",   "rsp",
        "r8",    "r9",    "r10",   "r11",   "r12",   "r13",   "r14",   "r15",
        "xmm0",  "xmm1",  "xmm2",  "xmm3",  "xmm4",  "xmm5",  "xmm6",  "xmm7",
        "xmm8",  "xmm9",  "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15",
        "xmm16", "xmm17", "xmm18", "xmm19", "xmm20", "xmm21", "xmm22", "xmm23",
        "xmm24", "xmm25", "xmm26", "xmm27", "xmm28", "xmm29", "xmm30", "xmm31"
        "cc", "fpsr", "eflags"
      : "volatile");
  }
}

#[path = "x86_common.rs"]
mod common;
