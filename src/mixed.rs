use crate::speed::Speed;


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
pub trait MixedClone<S: Speed>: Sized {
    /// Get a clone that could share some but not all semantically-important mutable state.
    ///
    /// Exact behavior is very implementation-dependent.
    ///
    /// Read [`MixedClone`] for more.
    #[must_use]
    fn mixed_clone(&self) -> Self;
}
