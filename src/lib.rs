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

#![no_std]

// #[cfg(feature = "std")]
// extern crate std;

// #[cfg(feature = "alloc")]
// extern crate alloc;


mod speed;

mod independent;
mod mirrored;
mod mixed;


pub use self::{independent::IndependentClone, mirrored::MirroredClone, mixed::MixedClone};
pub use self::speed::{Speed, NearInstant, ConstantTime, LogTime, AnySpeed};
