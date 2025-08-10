#![expect(clippy::absolute_paths, reason = "there's a lot of random types used")]
#![warn(clippy::missing_inline_in_public_items, reason = "almost everything is very short")]

use crate::call_varargs_macro;
use crate::speed::{Speed, NearInstant, ConstantTime, LogTime, AnySpeed};


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


macro_rules! non_recursive_near_instant {
    ($($({for $($bounds:tt)+})? $type:ty),* $(,)?) => {
        $(
            impl<$($($bounds)+)?> MirroredClone<NearInstant> for $type {
                #[inline]
                fn mirrored_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

non_recursive_near_instant! {
    (),
    core::convert::Infallible,
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
        impl<R, $($args),*> MirroredClone<NearInstant> for fn($($args),*) -> R {
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

make_tuple_macro!(tuple_constant, ConstantTime, $);
make_tuple_macro!(tuple_log, LogTime, $);
make_tuple_macro!(tuple_any, AnySpeed, $);

call_varargs_macro!(tuple_constant);
call_varargs_macro!(tuple_log);
call_varargs_macro!(tuple_any);

impl<S: Speed, T: MirroredClone<S>> MirroredClone<S> for Option<T> {
    #[inline]
    fn mirrored_clone(&self) -> Self {
        self.as_ref().map(T::mirrored_clone)
    }
}

impl<S: Speed, T: MirroredClone<S>, E: MirroredClone<S>> MirroredClone<S> for Result<T, E> {
    #[inline]
    fn mirrored_clone(&self) -> Self {
        self.as_ref()
            .map(T::mirrored_clone)
            .map_err(E::mirrored_clone)
    }
}
