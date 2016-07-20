// This file is part of libfringe, a low-level green threading library.
// Copyright (c) 2015, Nathan Zadoks <nathan@nathan7.eu>
// See the LICENSE file included in this distribution.


//! Helper types for cyclic protocols
//!
//! These types enforce that the given context take and yield values indefinitely, where the types
//! are recycled from a finite list. "indefinitely" because after any number of swaps, a thread may
//! terminate instead of yielding again.
//!
//! Concrete types are provided for periods up to 4 types long, but the trait will assist with
//! implementing cycles with longer periods.
use core::marker::PhantomData;

use void::{self, Void};

use stack::Stack;
use stack_pointer::StackPointer;
use super::context::*;
use super::safer_rebuild::*;

pub type Unpacked<'a, C: Cycle<'a>> = Context<'a, ArgsRaw<'a, C>, C::Stack>;
pub type ArgsRawest<C, Next, Arg> = Either<
    ExitedArgs<C, Arg>,
    YieldedArgs<Next, Arg>,
  >;
pub type ArgsRaw<'a, C: Cycle<'a>> = Either<
    ExitedArgs<C::Stack, <C as Cycle<'a>>::Arg>,
    YieldedArgs<C::Next, <C as Cycle<'a>>::Arg>,
  >;
pub type Args<'a, C: Cycle<'a>> = (Option<C::Next>, C::Arg);

fn pack_args<'a, C: Cycle<'a>>(args_raw: ArgsRaw<'a, C>) -> Args<'a, C> {
  match args_raw {
    Either::Left(ExitedArgs(arg, _)) => (None, arg),
    Either::Right(YieldedArgs(ctx, arg)) => (Some(ctx), arg),
  }
}

pub trait Cycle<'a>: Send + Sized + 'a
{
  type Arg: Send + 'a;
  type Next: Cycle<'a, Stack=Self::Stack>;
  type Stack: Stack + Send + 'a;

  #[inline(always)]
  fn pack(other: Unpacked<'a, Self>) -> Self;

  #[inline(always)]
  fn unpack(self) -> Unpacked<'a, Self>;

  #[inline(always)]
  #[inline(always)]
  fn new<F>(stack: Self::Stack, f: F) -> Self
    where F: FnOnce(&mut ThreadLocals<Self::Stack>, Args<'a, Self>)
                    -> Void + Send + 'a
  {
    Self::pack(Context::new(stack, |tl, a| f(tl, pack_args(a))))
  }

  #[inline(always)]
    fn swap(self,
            maybe_stack: Option<&'a mut ThreadLocals<Self::Stack>>,
            args: Self::Arg)
            -> Args<'a, Self::Next>
  {
    pack_args(self.unpack().switch_right(maybe_stack, args))
  }

  #[inline(always)]
  fn kontinue(self,
              maybe_stack: Option<&'a mut ThreadLocals<Self::Stack>>,
              args: Self::Arg)
              -> !
  {
    void::unreachable(self.unpack().switch_left(maybe_stack, args))
  }
}

pub struct ExitedArgs<A, S>(A, PhantomData<S>);
pub struct YieldedArgs<C, A>(C, A);

unsafe impl<'a, S, Arg> RebuildWithTl<'a> for ExitedArgs<S, Arg>
  where S: Stack + 'a + Send,
        Arg: 'a + Send,
{
  type Payload = Arg;
  type OldStack = S;
}

impl<'a, S: Stack + 'a, Arg: 'a + Send> Rebuild<'a> for ExitedArgs<S, Arg>
  where S: Stack + 'a + Send,
        Arg: 'a + Send,
{
  type OldArgs = Void;

  fn rebuild(ctx: Context<'a, Self::OldArgs, Self::OldStack>,
             arg: Self::Payload)
             -> Self
  {
    debug!("Switched from dead thread");
    drop(ctx);
    debug!("Dropped dead thread");
    ExitedArgs(arg, PhantomData)
  }
}

unsafe impl<'a, C: Cycle<'a>, Arg: 'a + Send> RebuildWithTl<'a> for YieldedArgs<C, Arg> {
  type Payload = Arg;
  type OldStack = C::Stack;
}

impl<'a, C: Cycle<'a>, Arg: 'a + Send> Rebuild<'a> for YieldedArgs<C, Arg> {
  type OldArgs = ArgsRaw<'a, C>;

  fn rebuild(ctx: Context<'a, Self::OldArgs, Self::OldStack>,
             arg: Self::Payload)
             -> Self
  {
    YieldedArgs(C::pack(ctx), arg)
  }
}



pub struct C1<'a, S, A>(pub Context<'a, ArgsRaw<'a, C1<'a, S, A>>, S>)
  where S: Stack + 'a + Send,
        A: 'a + Send;

impl<'a, S, A> Cycle<'a> for C1<'a, S, A>
  where S: Stack + Send + 'a,
        A: Send + 'a,
{
  type Arg = A;
  type Next = Self;
  type Stack = S;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C1(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}


/*
pub struct C2<'a, S, A, B>(pub Context<'a, (Option<C2<'a, S, B, A>>, A), S>)
  where S: Stack + 'a,
        A: 'a,
        B: 'a;

impl<'a, S, A, B> Cycle<'a> for C2<'a, S, A, B>
  where S: Stack + Send + 'a,
        A: Send + 'a,
        B: Send + 'a,
{
  type Arg = A;
  type Next = C2<'a, S, B, A>;
  type Stack = S;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C2(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}



pub struct C3<'a, S, A, B, C>(pub Context<'a, (Option<C3<'a, S, B, C, A>>, A), S>)
  where S: Stack + 'a,
        A: 'a,
        B: 'a,
        C: 'a;

impl<'a, S, A, B, C> Cycle<'a> for C3<'a, S, A, B, C>
  where S: Stack + Send + 'a,
        A: Send + 'a,
        B: Send + 'a,
        C: Send + 'a,
{
  type Arg = A;
  type Next = C3<'a, S, B, C, A>;
  type Stack = S;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C3(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}



pub struct C4<'a, S, A, B, C, D>(pub Context<'a, (Option<C4<'a, S, B, C, D, A>>, A), S>)
  where S: Stack + 'a,
        A: 'a,
        B: 'a,
        C: 'a,
        D: 'a;

impl<'a, S, A, B, C, D> Cycle<'a> for C4<'a, S, A, B, C, D>
  where S: Stack + Send + 'a,
        A: Send + 'a,
        B: Send + 'a,
        C: Send + 'a,
        D: Send + 'a,
{
  type Arg = A;
  type Next = C4<'a, S, B, C, D, A>;
  type Stack = S;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C4(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}
*/
