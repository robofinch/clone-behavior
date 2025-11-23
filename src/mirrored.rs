#![expect(clippy::absolute_paths, reason = "there's a lot of random types used")]
#![warn(clippy::missing_inline_in_public_items, reason = "almost everything is very short")]

use crate::call_varargs_macro;
use crate::speed::{Fast, MaybeSlow, Speed};


/// Get clones that share all semantically-important mutable state.
///
/// The goal is that mutating one mirrored clone affects every clone. Different mirrored clones
/// should, from their public interfaces, act identically, with some exceptions like memory
/// addresses.
///
/// This can be achieved via reference counting (e.g., [`Rc`] or [`Arc`]), or by simply
/// not having any mutable state to share at all (e.g. a zero-sized type like `()`, or a reference
/// to a type without internal mutability, like `&'static str`).
///
/// Per-clone mutable data is permissible, so long as the effects of mutating that data do not
/// cause mirrored clones to behave differently in a potentially-observable way.
/// Clones of types like `Option<T>` or `u32` are not considered to be mirrored clones, as
/// counterexamples.
///
/// # Exceptions
/// - A type implementing `MirroredClone` may specify that certain methods or accesses are not
///   subject to these guarantees.
/// - `Debug` should be assumed to be an exception. Intentionally worsening the life of someone
///   debugging your type is not a goal.
/// - Memory addresses which are different per-clone, but are not mutated by that clone (except
///   when a user moves the value). Trivially, mirrored clones do not have the same address,
///   and some of their data may be inlined into the type. If any reference to that data is exposed,
///   a user would likely be able to determine what data is inlined into the type and what is
///   inlined. That's already visible in the source code, anyway. Unless some semantically-important
///   function of the type depends on the address of it or its data, it's unimportant.
/// - TLDR of above bullet: users should not assume that references returned from different mirrored
///   clones refer to the same value, only that those values *behave* the same.
///
/// [`Rc`]: std::rc::Rc
/// [`Arc`]: std::sync::Arc
pub trait MirroredClone<S: Speed>: Sized {
    /// Get a clone that shares all semantically-important mutable state with its source.
    ///
    /// The goal is that mutating one mirrored clone affects every clone. Different mirrored clones
    /// should, from their public interfaces, act identically, with some exceptions like memory
    /// addresses.
    ///
    /// Read [`MirroredClone`] for more.
    #[must_use]
    fn mirrored_clone(&self) -> Self;
}

/// Quickly get clones that share all semantically-important mutable state.
///
/// See [`MirroredClone`] for more.
pub trait FastMirroredClone: MirroredClone<Fast> {
    /// Get a clone that shares all semantically-important mutable state with its source.
    ///
    /// The goal is that mutating one mirrored clone affects every clone. Different mirrored clones
    /// should, from their public interfaces, act identically, with some exceptions like memory
    /// addresses.
    ///
    /// See [`MirroredClone`] for more.
    #[must_use]
    fn fast_mirrored_clone(&self) -> Self;
}

impl<T: MirroredClone<Fast>> FastMirroredClone for T {
    #[inline]
    fn fast_mirrored_clone(&self) -> Self {
        self.mirrored_clone()
    }
}


macro_rules! non_recursive_fast {
    ($($({for $($bounds:tt)+})? $type:ty),* $(,)?) => {
        $(
            impl<S: Speed, $($($bounds)+)?> MirroredClone<S> for $type {
                #[inline]
                fn mirrored_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

non_recursive_fast! {
    (),
    {for T} core::iter::Empty<T>,
    {for T: ?Sized} core::marker::PhantomData<T>,
    core::marker::PhantomPinned,
    core::ops::RangeFull,
}

#[cfg(feature = "alloc")]
macro_rules! refcounted {
    ($($t:ident $refcounted:ty),* $(,)?) => {
        $(
            impl<S: Speed, $t: ?Sized> MirroredClone<S> for $refcounted {
                #[inline]
                fn mirrored_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

#[cfg(feature = "alloc")]
refcounted!(
    T alloc::rc::Rc<T>,
    T alloc::rc::Weak<T>,
    T alloc::sync::Arc<T>,
    T alloc::sync::Weak<T>,
);

macro_rules! function {
    ($($args:ident),*) => {
        impl<S: Speed, R, $($args),*> MirroredClone<S> for fn($($args),*) -> R {
            #[inline]
            fn mirrored_clone(&self) -> Self {
                *self
            }
        }
    };
}

macro_rules! pinned_refcounted {
    ($($t:ident $refcounted:ty),* $(,)?) => {
        $(
            #[cfg(feature = "alloc")]
            impl<S: Speed, $t: ?Sized> MirroredClone<S> for core::pin::Pin<$refcounted> {
                #[inline]
                fn mirrored_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

pinned_refcounted! {
    T alloc::rc::Rc<T>,
    T alloc::rc::Weak<T>,
    T alloc::sync::Arc<T>,
    T alloc::sync::Weak<T>,
}

function!();
call_varargs_macro!(function);

macro_rules! make_tuple_macro {
    ($name:ident, $speed:ident, $dollar:tt) => {
        macro_rules! $name {
            ($dollar($dollar args:ident),+) => {
                impl<$dollar($dollar args: MirroredClone<$speed>),+> MirroredClone<$speed>
                for ($dollar($dollar args,)+)
                {
                    #[inline]
                    fn mirrored_clone(&self) -> Self {
                        #[expect(
                            non_snake_case,
                            reason = "using `Tn` as the variable of type `Tn`",
                        )]
                        let ($dollar($dollar args,)+) = self;
                        (
                            $dollar($dollar args.mirrored_clone(),)+
                        )
                    }
                }
            };
        }
    };
}

make_tuple_macro!(tuple_fast, Fast, $);
make_tuple_macro!(tuple_slow, MaybeSlow, $);

call_varargs_macro!(tuple_fast);
call_varargs_macro!(tuple_slow);

impl<S: Speed> MirroredClone<S> for core::convert::Infallible {
    #[expect(
        clippy::missing_inline_in_public_items,
        clippy::uninhabited_references,
        reason = "this is unreachable",
    )]
    fn mirrored_clone(&self) -> Self {
        *self
    }
}
