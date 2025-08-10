#![expect(clippy::absolute_paths, reason = "there's a lot of random types used")]
#![warn(clippy::missing_inline_in_public_items, reason = "almost everything is very short")]

use crate::call_varargs_macro;
use crate::speed::{Speed, NearInstant, ConstantTime, LogTime, AnySpeed};


/// Get deep clones of a value, which do not share any semantically-important mutable state.
///
/// The goal is that the independent clone and its source appear to act completely independently,
/// at least from their public interfaces; mutating or dropping one clone (among possible actions)
/// should have no potentially-observable effect on any other independent clone.
///
/// Note also that an independent clone and its source may reference the same immutable data, even
/// if the public interface of the type could be used to confirm that two independent clones are
/// (or were) associated. (However, there should not be a way for either the source or any
/// independent clone to affect that data's value or address in memory.)
///
/// Side-channel attacks should be ignored in implementing this trait. The point is to provide deep
/// clones, not cryptographic-quality guarantees about how those deep clones are performed.
///
/// # Exceptions
/// - A type implementing `IndependentClone` may specify that certain methods or accesses are not
///   subject to these guarantees.
/// - `Debug` should be assumed to be an exception. Intentionally worsening the life of someone
///   debugging your type is not a goal.
/// - Shared mutable data which is written to but never read, or is read with effects that can only
///   be publicly observed through exceptions, is acceptable.
///
/// # Other edge cases
/// - A user calling `as_ptr()` and friends on references, to do address comparisons: if the
///   address returned could change, then even if the underlying value at a new address would be
///   the same, "the address of the data" should be considered mutable state. If that mutable state
///   can be affected by the actions of the source of the independent clone or by other clones,
///   then the address of the data must not be publicly exposed by the type's interface.
/// - Copy-on-write data: while likely permissible if properly encapsulated, presumably the source,
///   the independent clone, or both are going to be mutated; it'd be best to give the independent
///   clone a unique owned copy of the data. Further, if some sort of reference-counted
///   copy-on-write scheme is used where the last clone to refer to the data doesn't need to copy
///   it, then the reference count used to determine that is shared mutable state. This can be
///   avoided by simply giving the new independent clone an unique owned copy of the data, as
///   should likely be done anyway.
pub trait IndependentClone<S: Speed>: Sized {
    /// Get a deep clone of a value, which does not share any semantically-important mutable state.
    ///
    /// The goal is that the independent clone and its source appear to act completely
    /// independently, at least from their public interfaces; mutating or dropping one clone (among
    /// possible actions) should have no potentially-observable effect on any other independent
    /// clone.
    ///
    /// Read [`IndependentClone`] for more.
    #[must_use]
    fn independent_clone(&self) -> Self;
}


macro_rules! impl_copy {
    ($($types:ty),*) => {
        $(
            impl IndependentClone<NearInstant> for $types {
                #[inline]
                fn independent_clone(&self) -> Self {
                    *self
                }
            }
        )*
    };
}

macro_rules! int_impls {
    ($($num:ident),* $(,)?) => {
        $(
            impl_copy!($num, ::core::num::NonZero<$num>);
        )*
    };
}

int_impls!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
);

macro_rules! non_recursive_near_instant {
    ($($({for $($bounds:tt)+})? $type:ty),* $(,)?) => {
        $(
            impl<$($($bounds)+)?> IndependentClone<NearInstant> for $type {
                #[inline]
                fn independent_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

non_recursive_near_instant! {
    f32, f64, bool, char, (),
    core::alloc::Layout,
    core::any::TypeId,
    core::cmp::Ordering,
    core::convert::Infallible,
    {for T} core::iter::Empty<T>,
    {for T: ?Sized} core::marker::PhantomData<T>,
    core::marker::PhantomPinned,
    {for T} core::mem::Discriminant<T>,
    core::ops::RangeFull,
    core::sync::atomic::Ordering,
    core::time::Duration,
}

#[cfg(feature = "std")]
non_recursive_near_instant! {
    std::time::Instant,
    std::thread::ThreadId,
}

macro_rules! atomic {
    ($($name:ident $bits:literal),* $(,)?) => {
        $(
            #[cfg(target_has_atomic = $bits)]
            impl IndependentClone<NearInstant> for core::sync::atomic::$name {
                #[inline]
                fn independent_clone(&self) -> Self {
                    Self::new(self.load(core::sync::atomic::Ordering::Relaxed))
                }
            }
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

macro_rules! function {
    ($($args:ident),*) => {
        impl<R, $($args),*> IndependentClone<NearInstant> for fn($($args),*) -> R {
            #[inline]
            fn independent_clone(&self) -> Self {
                *self
            }
        }
    };
}

function!();
call_varargs_macro!(function);

macro_rules! make_tuple_macro {
    ($name:ident, $speed:ident, $dollar:tt) => {
        macro_rules! $name {
            ($dollar($dollar args:ident),+) => {
                impl<$dollar($dollar args: IndependentClone<$speed>),+> IndependentClone<$speed>
                for ($dollar($dollar args,)+)
                {
                    #[inline]
                    fn independent_clone(&self) -> Self {
                        #[expect(
                            non_snake_case,
                            reason = "using `Tn` as the variable of type `Tn`",
                        )]
                        let ($dollar($dollar args,)+) = self;
                        (
                            $dollar($dollar args.independent_clone(),)+
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

macro_rules! constant_or_slower {
    (
        $(
            $(#[$meta:meta])*
            $({for ($($special_bounds:ident)*) {$($where_bounds:tt)*} $($bounds:tt)*})?
            $type:ty
            {|$self:ident| $($body:tt)*}
        ),*
        $(,)?
    ) => {
        $(
            impl<$($($special_bounds: IndependentClone<ConstantTime>,)* $($bounds)*)?>
                IndependentClone<ConstantTime>
            for $type
            where
                $($($where_bounds)*)?
            {
                $(#[$meta])*
                #[inline]
                fn independent_clone(&$self) -> Self {
                    $($body)*
                }
            }

            impl<$($($special_bounds: IndependentClone<LogTime>,)* $($bounds)*)?>
                IndependentClone<LogTime>
            for $type
            where
                $($($where_bounds)*)?
            {
                $(#[$meta])*
                #[inline]
                fn independent_clone(&$self) -> Self {
                    $($body)*
                }
            }

            impl<$($($special_bounds: IndependentClone<AnySpeed>,)* $($bounds)*)?>
                IndependentClone<AnySpeed>
            for $type
            where
                $($($where_bounds)*)?
            {
                $(#[$meta])*
                #[inline]
                fn independent_clone(&$self) -> Self {
                    $($body)*
                }
            }
        )*
    };
}

constant_or_slower! {
    {for (T) {T: ?Sized} const N: usize} [T; N] {|self| {
        self.each_ref().map(T::independent_clone)
    }},
    {for (T) {}} Option<T> {|self| {
        self.as_ref().map(T::independent_clone)
    }},
    {for (T E) {}} Result<T, E> {|self| {
        self.as_ref()
            .map(T::independent_clone)
            .map_err(E::independent_clone)
    }},
    {for (T) {}} core::mem::ManuallyDrop<T> {|self| {
        Self::new(T::independent_clone(self))
    }},
    {for (T) {T: Copy}} core::cell::Cell<T> {|self| {
        Self::new(T::independent_clone(&self.get()))
    }},
    /// # Panics
    /// Panics if the value is currently mutably borrowed.
    {for (T) {}} core::cell::RefCell<T> {|self| {
        Self::new(T::independent_clone(&self.borrow()))
    }},
}

#[cfg(feature = "alloc")]
constant_or_slower! {
    {for (T) {T: ?Sized}} alloc::rc::Rc<T> {|self| {
        Self::new(T::independent_clone(self))
    }},
    {for (T) {T: ?Sized}} core::pin::Pin<alloc::rc::Rc<T>> {|self| {
        alloc::rc::Rc::pin(T::independent_clone(self))
    }},
    {for (T) {T: ?Sized}} alloc::rc::Weak<T> {|self| {
        if let Some(rc) = self.upgrade() {
            alloc::rc::Rc::downgrade(&alloc::rc::Rc::new(T::independent_clone(&rc)))
        } else {
            Self::new()
        }
    }},
    {for (T) {T: ?Sized}} alloc::sync::Arc<T> {|self| {
        Self::new(T::independent_clone(self))
    }},
    {for (T) {T: ?Sized}} core::pin::Pin<alloc::sync::Arc<T>> {|self| {
        alloc::sync::Arc::pin(T::independent_clone(self))
    }},
    {for (T) {T: ?Sized}} alloc::sync::Weak<T> {|self| {
        if let Some(arc) = self.upgrade() {
            alloc::sync::Arc::downgrade(&alloc::sync::Arc::new(T::independent_clone(&arc)))
        } else {
            Self::new()
        }
    }},
}

#[cfg(feature = "std")]
constant_or_slower! {
    /// # Panics
    /// Panics if the `RwLock` is poisoned.
    {for (T) {}} std::sync::RwLock<T> {|self| {
        let lock_result: Result<_, std::sync::PoisonError<_>> = self.read();
        #[expect(clippy::unwrap_used, reason = "Unwrapping poison")]
        Self::new(T::independent_clone(&lock_result.unwrap()))
    }},
    /// # Panics or Deadlocks
    /// Panics if the `Mutex` is poisoned.
    ///
    /// Will either panic or deadlock if the current thread already holds the mutex.
    {for (T) {}} std::sync::Mutex<T> {|self| {
        let lock_result: Result<_, std::sync::PoisonError<_>> = self.lock();
        #[expect(clippy::unwrap_used, reason = "Unwrapping poison")]
        Self::new(T::independent_clone(&lock_result.unwrap()))
    }},
}

// constant or slower: ranges, ops::Bound,

macro_rules! map_and_collect {
    ($($t:ident $({$($where_bounds:tt)*})? $type:ty),* $(,)?) => {
        $(
            #[cfg(feature = "alloc")]
            impl<$t: IndependentClone<AnySpeed>> IndependentClone<AnySpeed>
            for $type
            where
                $($($where_bounds)*)?
            {
                #[inline]
                fn independent_clone(&self) -> Self {
                    self.iter()
                        .map($t::independent_clone)
                        .collect()
                }
            }
        )*
    };
}

map_and_collect! {
    T alloc::boxed::Box<[T]>,
    T alloc::vec::Vec<T>,
    T alloc::collections::VecDeque<T>,
    T alloc::collections::LinkedList<T>,
    T {T: Ord} alloc::collections::BTreeSet<T>,
    T {T: Ord} alloc::collections::BinaryHeap<T>,
}

#[cfg(feature = "alloc")]
impl<T: IndependentClone<AnySpeed>> IndependentClone<AnySpeed>
for core::pin::Pin<alloc::boxed::Box<[T]>>
{
    #[inline]
    fn independent_clone(&self) -> Self {
        let new_box = self.iter()
            .map(T::independent_clone)
            .collect::<alloc::boxed::Box<[T]>>();

        alloc::boxed::Box::into_pin(new_box)
    }
}

#[cfg(feature = "alloc")]
impl IndependentClone<AnySpeed> for alloc::boxed::Box<str> {
    #[inline]
    fn independent_clone(&self) -> Self {
        self.clone()
    }
}

#[cfg(feature = "alloc")]
impl IndependentClone<AnySpeed> for core::pin::Pin<alloc::boxed::Box<str>> {
    #[inline]
    fn independent_clone(&self) -> Self {
        self.clone()
    }
}

#[cfg(feature = "alloc")]
impl<K, V> IndependentClone<AnySpeed> for alloc::collections::BTreeMap<K, V>
where
    K: IndependentClone<AnySpeed> + Ord,
    V: IndependentClone<AnySpeed>,
{
    #[inline]
    fn independent_clone(&self) -> Self {
        self.iter()
            .map(|(key, val)| {
                (
                    K::independent_clone(key),
                    V::independent_clone(val),
                )
            })
            .collect()
    }
}

#[cfg(feature = "std")]
impl<T, S> IndependentClone<AnySpeed> for std::collections::HashSet<T, S>
where
    T: IndependentClone<AnySpeed> + Eq + core::hash::Hash,
    S: core::hash::BuildHasher + Default,
{
    #[inline]
    fn independent_clone(&self) -> Self {
        self.iter()
            .map(T::independent_clone)
            .collect()
    }
}

#[cfg(feature = "std")]
impl<K, V, S> IndependentClone<AnySpeed> for std::collections::HashMap<K, V, S>
where
    K: IndependentClone<AnySpeed> + Eq + core::hash::Hash,
    V: IndependentClone<AnySpeed>,
    S: core::hash::BuildHasher + Default,
{
    #[inline]
    fn independent_clone(&self) -> Self {
        self.iter()
            .map(|(key, val)| {
                (
                    K::independent_clone(key),
                    V::independent_clone(val),
                )
            })
            .collect()
    }
}

// TODO: iterators, other boxed things
