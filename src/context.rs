// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.
use core::marker::{PhantomData, Unsize};
use core::ptr::Unique;

use void::Void;

use arch::Registers;
use stack;
use debug::StackId;

/// Context is the raw heart of libfringe. A context represents a
/// paused thread of execution. The computation within a context is
/// assumed to diverge--more syncronous abstractions are build on top.
///
/// The context is scoped in that a child context only valid for a
/// lifetime (aka borrows something from its parent) must eventually
/// return to it's parent.
///
/// I *think* the lifetimes here might enforce that
///
//#[derive(Debug)]
// TODO Higher-kinded-lifetime with `Args`?
pub struct Context<'a, Args: 'a> {
  regs: Registers,
  pub thread_locals: Option<Unique<ThreadLocals<stack::Stack + 'a>>>,
  _ref: PhantomData<&'a mut fn(Args) -> !>
}

/// This is the type we actual exchange between Contexts.
///
/// The closure is used because the arguments may depend on the old
/// stack pointer.
///
/// The closure is dynamic so as to requre less knowledge on the
/// receiving thread. That requirement might have been so onerous so
/// as to force static schedualing between the contexts.
///
/// FnRaw is like FnBox
type Abi<'a, Args: 'a> = Unique<FnRaw(Registers) -> Args + 'a>;


// TODO require that static is also Send
unsafe impl<'a, Args: Send> Send for Context<'a, Args> {}

impl<'a, Args> Context<'a, Args>
  where Context<'a, Args>: Send,
        Args: 'a
{
  /// Create a new Context. When it is swapped into,
  /// it will call the passed closure.
  /// Not exception safe [You can catch unwinding manually]
  #[inline(always)]
  pub fn new<S, F>(mut stack: S, f: F) -> Self
    where S: stack::Stack + Send + 'a,
          F: FnOnce(&mut ThreadLocals<S>, Args) -> Void + Send + 'a
  {
    let top = stack.top();
    let tl = ThreadLocals {
      _stack_id: StackId::register(&mut stack),
      stack: stack
    };
    let (regs, fp) = unsafe { Registers::fat_new(top, InitClosure(tl, f)) };
    Context {
      regs: regs,
      thread_locals: Some(unsafe { Unique::new(&mut (*fp).0 as *mut _) }),
      _ref: PhantomData
    }
  }
}

impl<'a, TheirArgs> Context<'a, TheirArgs>
  where 'a,
        Context<'a, TheirArgs>: Send + 'a,
        TheirArgs: Send + 'a,
{
  #[inline(always)]
  unsafe fn raw_switch<'b, OurArgs, F>(self, mut argsf: F) -> OurArgs
    where 'a: 'b,
          F: FnOnce(Registers) -> TheirArgs + Send + 'b,
          OurArgs: Send + 'b
  {
    let args: Abi<'b, TheirArgs> = Unique::new(&mut argsf as *mut _ as *mut _);
    debug!("new regs: {:?}", self.regs);
    let (regs, argsf2) =
      Registers::fat_swap::<Abi<'b, TheirArgs>, Abi<'b, OurArgs>>(self.regs, args);
    debug!("old regs: {:?}", regs);
    ::core::mem::forget((self, argsf));
    marshall(regs, argsf2)
  }

  #[inline(always)]
  pub fn switch<'b, OurArgs, F, S>
    (self,
     maybe_stack: Option<&mut ThreadLocals<S>>,
     f: F)
     -> OurArgs
    where S: Unsize<stack::Stack> + Send,
          F: FnOnce(Context<'b, OurArgs>) -> TheirArgs + Send + 'a,
          Context<'b, OurArgs>: Send + 'b,
          OurArgs: Send + 'b,
  {
    unsafe {
      self.raw_switch(|regs| {
        let ctx = Context {
          regs:          regs,
          thread_locals: maybe_stack.map(|tl| Unique::new(tl as *mut _ as *mut _)),
          _ref:          PhantomData,
        };
        f(ctx)
      })
    }
  }
}

impl<'a, 'b, TheirArgs, OurArgs> Context<'a, (Context<'b, OurArgs>, TheirArgs)>
  where 'b: 'a,
        Context<'a, (Context<'b, OurArgs>, TheirArgs)>: Send + 'a,
        Context<'b, OurArgs>: Send + 'b,
        TheirArgs: Send + 'a,
        OurArgs:   Send + 'b
{
  #[inline(always)]
  pub fn callcc<S>(self,
                   maybe_stack: Option<&mut ThreadLocals<S>>,
                   args: TheirArgs)
                   -> OurArgs
    where S: Unsize<stack::Stack> + Send
  {
    self.switch(maybe_stack, |ctx| (ctx, args))
  }
}

impl<'a, Args> Drop for Context<'a, Args> {
  /// Abandon the given context, dropping the stack it contained.
  #[inline]
  fn drop(&mut self) {
    if let Some(ref ptr) = self.thread_locals {
      unsafe { ::core::intrinsics::drop_in_place(**ptr) }
    }
  }
}


#[inline(always)]
unsafe fn marshall<'a, Args: 'a>(regs: Registers, fp: Abi<'a, Args>) -> Args {
  let fp2: &'a mut FnRaw(Registers) -> Args =
    ::core::mem::transmute(*fp as *mut _);
  fp2.call_raw((regs,)) //panic!()
}

trait FnRaw<A> {
  type Output;
  unsafe fn call_raw(&mut self, args: A) -> Self::Output;
}

impl<A, F> FnRaw<A> for F where F: FnOnce<A> {
  type Output = <Self as FnOnce<A>>::Output;
  unsafe fn call_raw(&mut self, args: A) -> Self::Output {
    ::core::ptr::read(self as *mut Self).call_once(args)
  }
}


/// The stack is owned by itself
#[derive(Debug)]
pub struct ThreadLocals<Stack: ?Sized> {
  _stack_id: StackId,
  stack: Stack
}

pub const NATIVE_THREAD_LOCALS: Option<&'static mut ThreadLocals<Void>> = None;


/// We need to reference thing in closure, hence manual impl
struct InitClosure<S, F>(ThreadLocals<S>, F);

impl<'a, S, F, Args: 'a> FnOnce<(Registers, Abi<'a, Args>)> for InitClosure<S, F>
  where S: stack::Stack + Send + 'a,
        F: FnOnce(&mut ThreadLocals<S>, Args) -> Void + Send + 'a,
        InitClosure<S, F>: Send + 'a,
{
  type Output = Void;

  extern "rust-call" fn call_once(mut self,
                                  (regs, argsf): (Registers, Abi<'a, Args>))
                                  -> Void
  {
    self.1(&mut self.0, unsafe { marshall(regs, argsf) })
  }
}
