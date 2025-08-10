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

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;


mod speed;

mod independent;
mod mirrored;
mod mixed;

mod blanket_impls;


pub use self::{
    blanket_impls::NonRecursive,
    independent::IndependentClone,
    mirrored::MirroredClone,
    mixed::MixedClone,
};
pub use self::speed::{Speed, NearInstant, ConstantTime, LogTime, AnySpeed};


macro_rules! call_varargs_macro {
    ($macro:ident) => {
        $macro!(T1);
        $macro!(T1, T2);
        $macro!(T1, T2, T3);
        $macro!(T1, T2, T3, T4);
        $macro!(T1, T2, T3, T4, T5);
        $macro!(T1, T2, T3, T4, T5, T6);
        $macro!(T1, T2, T3, T4, T5, T6, T7);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
    };
}

pub(crate) use call_varargs_macro;
