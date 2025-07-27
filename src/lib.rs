// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/generic-container/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/generic-container/blob/main/LICENSE-MIT
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![expect(
    missing_copy_implementations,
    missing_debug_implementations,
    reason = "The four uninhabited Speed types are what trigger these lints",
)]

mod sealed {
    #[expect(unnameable_types, reason = "This is intentional, and creates a sealed trait")]
    pub trait Sealed {}
}


use self::sealed::Sealed;


/// Trait for indicating the overhead and/or time complexity of a cloning operation.
pub trait Speed: Sealed {}

/// Indicates that a cloning operation is:
/// - constant time
/// - nonblocking (atomic operations may be fine, but acquiring a lock is not)
/// - at most the overhead of a few atomic operations
/// - generally, deserving of being called near-instant in speed.
///
/// The nightly-only [`UseCloned`](https://doc.rust-lang.org/std/clone/trait.UseCloned.html) trait
/// provides additional constraints, that may be worth considering.
pub enum NearInstant {}
/// Indicates that a cloning operation is constant-time, but might involve acquiring a lock or
/// performing some computations that aren't [`NearInstant`].
pub enum ConstantTime {}
/// Indicates that a cloning operation operates in logarithmic time or faster.
pub enum LogTime {}
/// Places no constraint on the overhead or time complexity of a cloning operation.
pub enum AnySpeed {}

impl Speed for NearInstant {}
impl Speed for ConstantTime {}
impl Speed for LogTime {}
impl Speed for AnySpeed {}

impl Sealed for NearInstant {}
impl Sealed for ConstantTime {}
impl Sealed for LogTime {}
impl Sealed for AnySpeed {}


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

/// Get clones that share all semantically-important mutable state.
///
/// The goal is that mutating one mirrored clone affects every clone. Different mirrored clones
/// should, from their public interfaces, act identically, with some exceptions like memory
/// addresses.
///
/// This can be achieved via reference counting (e.g., [`Rc`] or [`Arc`]), or by simply
/// not having any mutable state to share at all.
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

/// Get clones that could share some but not all semantically-important mutable state.
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
    /// Read [`MixedClone`] for more.
    #[must_use]
    fn mixed_clone(&self) -> Self;
}
