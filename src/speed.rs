mod sealed {
    #[expect(unnameable_types, reason = "This is intentional, and creates a sealed trait")]
    pub trait Sealed {}
}


use self::sealed::Sealed;


/// Indicates that a cloning operation is constant-time.
///
/// This might be nearly instant, or involve briefly acquiring a lock and performing a few
/// computations.
pub enum Fast {}
/// Places no constraints on the speed of a cloning operation.
pub enum MaybeSlow {}


impl Sealed for Fast {}
impl Sealed for MaybeSlow {}


/// Trait for indicating the overhead and/or time complexity of a cloning operation.
pub trait Speed: Sealed {}

impl Speed for Fast {}
impl Speed for MaybeSlow {}
