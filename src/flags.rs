use bitflags::bitflags;

bitflags! {
    /// Bit masks for non-dynamic drawable flags.
    #[repr(transparent)]
    pub struct ConstantFlags: u8 {
        /// Additive blend mode mask.
        const BLEND_ADDITIVE = cubism_core_sys::csmBlendAdditive as _;
        /// Multiplicative blend mode mask.
        const BLEND_MULTIPLICATIVE = cubism_core_sys::csmBlendMultiplicative as _;
        /// Double-sidedness mask.
        const IS_DOUBLE_SIDED = cubism_core_sys::csmIsDoubleSided as _;
        /// Clipping mask inversion mode mask.
        const IS_INVERTED_MASK = cubism_core_sys::csmIsInvertedMask as _;
    }
}

impl ConstantFlags {
    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        (self.bits() & !Self::all().bits()) == 0
    }
}

bitflags! {
    /// Bit masks for dynamic drawable flags.
    #[repr(transparent)]
    pub struct DynamicFlags: u8 {
        /// Flag set when visible.
        const IS_VISIBLE = cubism_core_sys::csmIsVisible as _;
        /// Flag set when visibility did change.
        const VISIBILITY_DID_CHANGE = cubism_core_sys::csmVisibilityDidChange as _;
        /// Flag set when opacity did change.
        const OPACITY_DID_CHANGE = cubism_core_sys::csmOpacityDidChange as _;
        /// Flag set when draw order did change.
        const DRAW_ORDER_DID_CHANGE = cubism_core_sys::csmDrawOrderDidChange as _;
        /// Flag set when render order did change.
        const RENDER_ORDER_DID_CHANGE = cubism_core_sys::csmRenderOrderDidChange as _;
        /// Flag set when vertex positions did change.
        const VERTEX_POSITIONS_DID_CHANGE = cubism_core_sys::csmVertexPositionsDidChange as _;
    }
}

impl DynamicFlags {
    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        (self.bits() & !Self::all().bits()) == 0
    }
}
