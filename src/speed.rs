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
