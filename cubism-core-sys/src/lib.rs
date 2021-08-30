#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[cfg(not(feature = "doc"))]
include!(concat!(env!("OUT_DIR"), "/cubism_core.rs"));

#[cfg(feature = "doc")]
include!("../bindgen/cubism_core.rs");

/// Cubism moc
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct csmMoc {
    _unused: [u8; 0],
}

/// Cubism model
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct csmModel {
    _unused: [u8; 0],
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::align_of;

    #[test]
    fn test_alignment() {
        assert_eq!(align_of::<csmMoc>(), csmAlignofMoc as usize);
        assert_eq!(align_of::<csmModel>(), csmAlignofModel as usize);
    }
}
