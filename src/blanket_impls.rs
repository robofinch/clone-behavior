#![expect(clippy::absolute_paths, reason = "there's a lot of random types used")]

use crate::call_varargs_macro;
use crate::{independent::IndependentClone, mirrored::MirroredClone, mixed::MixedClone};
use crate::speed::{NearInstant, ConstantTime, LogTime, AnySpeed};


/// Indicates that the speed of cloning a type does not recursively depend on any generics, opting
/// the type into several blanket implementations.
///
/// As examples, `u8` and `Rc<T>` implement this trait, while `Option<T>` does not.
///
/// A `NonRecursive` type need only implement a cloning operation at the fastest applicable speed,
/// and blanket implementations handle the rest. These blanket implementations would interfere
/// with, for example, an attempt to pass through the speed of cloning `T` to the speed of cloning
/// `Option<T>`.
pub trait NonRecursive {}


macro_rules! blanket_impls {
    ($clone_tr:ident, $clone_fn:ident) => {
        impl<T: NonRecursive + $clone_tr<NearInstant>> $clone_tr<ConstantTime> for T {
            #[inline]
            fn $clone_fn(&self) -> Self {
                <T as $clone_tr<NearInstant>>::$clone_fn(self)
            }
        }

        impl<T: NonRecursive + $clone_tr<ConstantTime>> $clone_tr<LogTime> for T {
            #[inline]
            fn $clone_fn(&self) -> Self {
                <T as $clone_tr<ConstantTime>>::$clone_fn(self)
            }
        }

        impl<T: NonRecursive + $clone_tr<LogTime>> $clone_tr<AnySpeed> for T {
            #[inline]
            fn $clone_fn(&self) -> Self {
                <T as $clone_tr<LogTime>>::$clone_fn(self)
            }
        }
    };
}

blanket_impls!(IndependentClone, independent_clone);
blanket_impls!(MirroredClone, mirrored_clone);
blanket_impls!(MixedClone, mixed_clone);


macro_rules! int_impls {
    ($($num:ident),* $(,)?) => {
        $(
            impl NonRecursive for $num {}
            impl NonRecursive for ::core::num::NonZero<$num> {}
        )*
    };
}

int_impls!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
);

macro_rules! non_recursive {
    ($($({for $($bounds:tt)+})? $type:ty),* $(,)?) => {
        $(
            impl<$($($bounds)+)?> NonRecursive for $type {}
        )*
    };
}

non_recursive! {
    f32, f64, bool, char, (),
    {for T: ?Sized} &T,
    {for T: ?Sized} *const T,
    {for T: ?Sized} *mut T,
    core::alloc::Layout,
    core::any::TypeId,
    core::cmp::Ordering,
    core::convert::Infallible,
    {for T} core::iter::Empty<T>,
    {for T: ?Sized} core::marker::PhantomData<T>,
    core::marker::PhantomPinned,
    {for T} core::mem::Discriminant<T>,
    core::ops::RangeFull,
    {for T: ?Sized} core::ptr::NonNull<T>,
    core::sync::atomic::Ordering,
    core::time::Duration,
}

#[cfg(feature = "alloc")]
non_recursive! {
    alloc::string::String,
}

#[cfg(feature = "std")]
non_recursive! {
    std::path::Path,
    std::path::PathBuf,
    std::time::Instant,
    std::thread::ThreadId,
}

macro_rules! atomic {
    ($($name:ident $bits:literal),* $(,)?) => {
        $(
            #[cfg(target_has_atomic = $bits)]
            impl NonRecursive for core::sync::atomic::$name {}
        )*
    };
}

atomic! {
    AtomicBool    "8",
    AtomicI8      "8", AtomicU8      "8",
    AtomicI16    "16", AtomicU16    "16",
    AtomicI32    "32", AtomicU32    "32",
    AtomicI64    "64", AtomicU64    "64",
    AtomicIsize "ptr", AtomicUsize "ptr",
}

#[cfg(target_has_atomic = "ptr")]
impl<T> NonRecursive for core::sync::atomic::AtomicPtr<T> {}

macro_rules! function {
    ($($args:ident),*) => {
        impl<R, $($args),*> NonRecursive for fn($($args),*) -> R {}
    };
}

function!();
call_varargs_macro!(function);
