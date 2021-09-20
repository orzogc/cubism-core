use bitflags::bitflags;

bitflags! {
    /// Bit masks for the static drawable flags.
    #[repr(transparent)]
    pub struct ConstantFlags: u8 {
        /// Additive blend mode mask.
        const BLEND_ADDITIVE = cubism_core_sys::csmBlendAdditive as _;
        /// Multiplicative blend mode mask.
        const BLEND_MULTIPLICATIVE = cubism_core_sys::csmBlendMultiplicative as _;
        /// Double-sidedness mask.
        const IS_DOUBLE_SIDED = cubism_core_sys::csmIsDoubleSided as _;
        /// Inversion mode mask.
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
    /// Bit masks for the dynamic drawable flags.
    #[repr(transparent)]
    pub struct DynamicFlags: u8 {
        /// A bit is set when the drawable is displayed.
        const IS_VISIBLE = cubism_core_sys::csmIsVisible as _;
        /// A bit is raised when [`IS_VISIBLE`](DynamicFlags::IS_VISIBLE) has been changed from the previous state.
        const VISIBILITY_DID_CHANGE = cubism_core_sys::csmVisibilityDidChange as _;
        /// A bit is raised when the opacity of a drawable has been changed.
        const OPACITY_DID_CHANGE = cubism_core_sys::csmOpacityDidChange as _;
        /// A bit is raised when the draw order of a drawable has been changed.
        const DRAW_ORDER_DID_CHANGE = cubism_core_sys::csmDrawOrderDidChange as _;
        /// A bit is raised when the rendering order of a drawable has been changed.
        const RENDER_ORDER_DID_CHANGE = cubism_core_sys::csmRenderOrderDidChange as _;
        /// A bit is raised when the vertex positions of a drawable has been changed.
        const VERTEX_POSITIONS_DID_CHANGE = cubism_core_sys::csmVertexPositionsDidChange as _;
    }
}

impl DynamicFlags {
    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        (self.bits() & !Self::all().bits()) == 0
    }
}
