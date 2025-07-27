use crate::speed::Speed;


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
