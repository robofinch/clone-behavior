#![expect(clippy::absolute_paths, reason = "there's a lot of random types used")]
#![warn(clippy::missing_inline_in_public_items, reason = "almost everything is very short")]

use crate::speed::{Speed, NearInstant};


/// Get clones that could share some but not all semantically-important mutable state.
///
/// This marker is useful for cases where a user needs to check documentation to figure out what
/// happens.
///
/// It is not *strictly* necessary that the clone function doesn't behave like
/// [`IndependentClone::independent_clone`] or [`MirroredClone::mirrored_clone`], but this type is
/// not a catch-all for "some sort of clone, maybe also implements [`IndependentClone`] or
/// [`MirroredClone`]".
///
/// This crate *will* provide a better tool for abstracting over all three modes of cloning
/// provided. It just isn't provided yet. This isn't it.
///
/// [`IndependentClone`]: crate::IndependentClone
/// [`IndependentClone::independent_clone`]: crate::IndependentClone::independent_clone
/// [`MirroredClone`]: crate::MirroredClone
/// [`MirroredClone::mirrored_clone`]: crate::MirroredClone::mirrored_clone
pub trait MixedClone<S: Speed>: Sized {
    /// Get a clone that could share some but not all semantically-important mutable state.
    ///
    /// Exact behavior is very implementation-dependent.
    ///
    /// Read [`MixedClone`] for more.
    #[must_use]
    fn mixed_clone(&self) -> Self;
}


macro_rules! non_recursive_near_instant {
    ($($({for $($bounds:tt)+})? $type:ty),* $(,)?) => {
        $(
            impl<$($($bounds)+)?> MixedClone<NearInstant> for $type {
                #[inline]
                fn mixed_clone(&self) -> Self {
                    self.clone()
                }
            }
        )*
    };
}

non_recursive_near_instant! {
    {for T: ?Sized} &T,
    {for T: ?Sized} *const T,
    {for T: ?Sized} *mut T,
    {for T: ?Sized} core::ptr::NonNull<T>,
}

#[cfg(target_has_atomic = "ptr")]
impl<T> MixedClone<NearInstant> for core::sync::atomic::AtomicPtr<T> {
    #[inline]
    fn mixed_clone(&self) -> Self {
        Self::new(self.load(core::sync::atomic::Ordering::Relaxed))
    }
}

#[cfg(feature = "alloc")]
impl<S: Speed, T: ?Sized + alloc::borrow::ToOwned> MixedClone<S> for alloc::borrow::Cow<'_, T> {
    #[inline]
    fn mixed_clone(&self) -> Self {
        self.clone()
    }
}

// TODO: consider implementing ones with recursive constraints
