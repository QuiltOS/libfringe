// This file is part of libfringe, a low-level green threading library.
// Copyright (c) Nathan Zadoks <nathan@nathan7.eu>,
//               John Ericson <Ericson2314@Yahoo.com>
// See the LICENSE file included in this distribution.
use core::marker::PhantomData;
use core::ptr::Unique;

use void::Void;

use stack::Stack;
use stack_pointer::StackPointer;
use debug::StackId;
use fat_args;


/// The `Context` is a rough equivalent to fringe's main `Context`, but serves
/// only as an implementation aid.
///
/// The context is scoped in that a child context only valid for a
/// lifetime (aka borrows something from its parent) must eventually
/// return to it's parent.
///
/// I *think* the lifetimes here might enforce that

// TODO Higher-kinded-lifetime with `ArgsF::Output`?
//#[derive(Debug)]
pub struct Context<'a, Args, S>
  where Args: /*RebuildRaw<'a> +*/ 'a,
        S: Stack + 'a,
{
  pub(super) stack_pointer: StackPointer,
  pub thread_locals: Option<Unique<ThreadLocals<S>>>,
  pub(super) _ref: PhantomData<&'a mut fn(StackPointer, Args/*::Payload*/) -> !>
}

// TODO require that static is also Send
unsafe impl<'a, Args, S> Send for Context<'a, Args, S>
  where Args: /*RebuildRaw<'a> +*/ 'a + Send,
        S: Stack + 'a + Send,
        //Args::Payload: Send,
{ }

impl<'a, Args, S> Context<'a, Args, S>
  where Context<'a, Args, S>: Send,
        Args: RebuildRaw<'a> + 'a + Send,
        S: Stack + 'a,
{
  /// Create a new Context. When it is swapped into, it will call the passed
  /// closure.
  #[inline(always)]
  pub fn new<F>(mut stack: S, fun: F) -> Self
    where F: FnOnce(&mut ThreadLocals<S>, Args) -> Void + Send + 'a
  {
    let mut sp = StackPointer::new(&stack);
    let tl = ThreadLocals {
      _stack_id: StackId::register(&mut stack),
      stack: stack
    };
    let tlp = unsafe {
      fat_args::init(&mut sp, move |initializer_sp| {
        // We explicitly move the thread-locals and closure so they are owned by
        // the stack
        let mut tl_move = tl;
        let fun_move = fun;
        // Yield back to context, given whoever initialized us nothing and
        // expecting a `ArgsF` to assemble our `ArgsF::Output`
        let (sp, payload) = fat_args::swap(initializer_sp,
                                           Unique::new(&mut tl_move));
        let args = Args::rebuild_raw(sp, payload);
        fun_move(&mut tl_move, args)
      })
    };
    Context {
      stack_pointer: sp,
      thread_locals: Some(tlp),
      _ref: PhantomData
    }
  }
}

impl<'a, TheirArgs, TheirStack> Context<'a, TheirArgs, TheirStack>
  where Context<'a, TheirArgs, TheirStack>: Send + 'a,
        TheirArgs: RebuildRaw<'a> + 'a,
        TheirStack: Stack + 'a,
{
  #[inline(always)]
  pub(super) unsafe fn raw_switch<'b, OurArgs, OurStack>
    (self, their_payload: TheirArgs::PayloadRaw) -> OurArgs
    where 'b: 'a,
          Context<'a, OurArgs, OurStack>: Send + 'b,
          OurArgs: RebuildRaw<'a> + 'a,
          OurStack: Stack + 'b,
  {
    debug!("new stack_pointer: {:?}", self.stack_pointer);
    let (sp, our_payload) =
      fat_args::swap(self.stack_pointer.clone(), their_payload);
    debug!("old stack_pointer: {:?}", sp);
    ::core::mem::forget(self);
    OurArgs::rebuild_raw(sp, our_payload)
  }
}

/// Session contexts must rebuild the argument from a black-box payload and the
/// old stack pointer before as the last step before handing off control to the
/// new coroutine. This trait describes how to do that.
pub unsafe trait RebuildRaw<'a> {
  /// The extra data sent over in addition to the old stack pointer
  type PayloadRaw: 'a;

  /// The function which actually does the rebuilding.
  unsafe fn rebuild_raw(StackPointer, Self::PayloadRaw) -> Self where Self: 'a;
}

unsafe impl<'a> RebuildRaw<'a> for Void
{
  type PayloadRaw = Void;
  unsafe fn rebuild_raw(sp: StackPointer, payload: Void) -> Void { payload }
}

impl<'a, Args, S> Drop for Context<'a, Args, S>
  where Args: /*RebuildRaw<'a> +*/ 'a,
        S: Stack + 'a,
{
  /// Abandon the given context, dropping the stack it contained.
  #[inline]
  fn drop(&mut self) {
    if let Some(ref ptr) = self.thread_locals {
      unsafe { ::core::intrinsics::drop_in_place(**ptr) }
    }
  }
}

/// The stack is owned by itself
#[derive(Debug)]
pub struct ThreadLocals<Stack: ?Sized> {
  _stack_id: StackId,
  stack: Stack
}

pub const NATIVE_THREAD_LOCALS: Option<&'static mut ThreadLocals<Void>> = None;
