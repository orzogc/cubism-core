//! This crate provides low level bindings to [Cubism Core native API](https://www.live2d.com/en/download/cubism-sdk/download-native/).
//!
//! For a safe wrapper, see the `cubism-core` crate.

#![warn(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(deref_nullptr)]

#[cfg(not(feature = "doc"))]
include!(concat!(env!("OUT_DIR"), "/cubism_core.rs"));

#[cfg(feature = "doc")]
include!("../bindgen/cubism_core.rs");

/// Cubism moc.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct csmMoc {
    _unused: [u8; 0],
}

/// Cubism model.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct csmModel {
    _unused: [u8; 0],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_alignment() {
        assert_eq!(mem::align_of::<csmMoc>(), csmAlignofMoc as _);
        assert_eq!(mem::align_of::<csmModel>(), csmAlignofModel as _);
    }
}
