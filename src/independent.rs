#![warn(clippy::missing_inline_in_public_items, reason = "almost everything is very short")]

use crate::speed::Speed;


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
