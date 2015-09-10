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

use core::marker::Unsize;

use void::{self, Void};

use stack;
use context::*;

pub type Unpacked<'a, C: Cycle<'a>> = Context<'a, Args<'a, C>>;
pub type Args<'a, C: Cycle<'a>> = (Option<<C as Cycle<'a>>::Next>, <C as Cycle<'a>>::Arg);

pub trait Cycle<'a>: Send + Sized + 'a {
  type Arg: Send;
  type Next: Cycle<'a>;

  #[inline(always)]
  fn pack(other: Unpacked<'a, Self>) -> Self;

  #[inline(always)]
  fn unpack(self) -> Unpacked<'a, Self>;

  #[inline(always)]
  #[inline(always)]
  fn new<S, F>(stack: S, f: F) -> Self
    where S: stack::Stack + Send + 'a,
          F: FnOnce(&mut ThreadLocals<S>, Args<'a, Self>) -> Void + Send + 'a
  {
    Self::pack(Context::new(stack, f))
  }

  #[inline(always)]
  fn swap<S>(self,
             maybe_stack: Option<&mut ThreadLocals<S>>,
             args: Self::Arg)
             -> Args<'a, Self::Next>
    where S: Unsize<stack::Stack> + Send
  {
    self.unpack().switch(maybe_stack, |ctx| (Some(Cycle::pack(ctx)), args))
  }

  #[inline(always)]
  fn kontinue<S>(self,
                 maybe_stack: Option<&mut ThreadLocals<S>>,
                 args: Self::Arg)
                 -> !
    where S: Unsize<stack::Stack> + Send
  {
    void::unreachable(self.unpack().switch(maybe_stack, |ctx| {
      debug!("Switched from dead thread");
      drop(ctx);
      debug!("Dropped dead thread");
      (None, args)
    }))
  }
}



pub struct C1<'a, A>(pub Context<'a, (Option<C1<'a, A>>, A)>)
  where A: 'a;

impl<'a, A> Cycle<'a> for C1<'a, A>
  where A: Send + 'a,
{
  type Arg = A;
  type Next = Self;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C1(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}



pub struct C2<'a, A, B>(pub Context<'a, (Option<C2<'a, B, A>>, A)>)
  where A: 'a,
        B: 'a;

impl<'a, A, B> Cycle<'a> for C2<'a, A, B>
  where A: Send + 'a,
        B: Send + 'a
{
  type Arg = A;
  type Next = C2<'a, B, A>;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C2(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}



pub struct C3<'a, A, B, C>(pub Context<'a, (Option<C3<'a, B, C, A>>, A)>)
  where A: 'a,
        B: 'a,
        C: 'a;

impl<'a, A, B, C> Cycle<'a> for C3<'a, A, B, C>
  where A: Send + 'a,
        B: Send + 'a,
        C: Send + 'a
{
  type Arg = A;
  type Next = C3<'a, B, C, A>;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C3(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}



pub struct C4<'a, A, B, C, D>(pub Context<'a, (Option<C4<'a, B, C, D, A>>, A)>)
  where A: 'a,
        B: 'a,
        C: 'a,
        D: 'a;

impl<'a, A, B, C, D> Cycle<'a> for C4<'a, A, B, C, D>
  where A: Send + 'a,
        B: Send + 'a,
        C: Send + 'a,
        D: Send + 'a
{
  type Arg = A;
  type Next = C4<'a, B, C, D, A>;

  fn pack(other: Unpacked<'a,  Self>) -> Self { C4(other) }
  fn unpack(self) -> Unpacked<'a, Self> { self.0 }
}
