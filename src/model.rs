use crate::{
    drawable::{DynamicDrawables, StaticDrawables},
    parameter::StaticParameters,
    part::StaticParts,
    Error, Moc, Result, ALIGN_OF_MODEL, {ConstantFlags, DynamicFlags},
};
use aligned_utils::bytes::AlignedBytes;
use std::{collections::HashMap, ffi::CStr, mem, slice};

const ISIZE_MAX: usize = isize::MAX as _;
const I32_MAX: u32 = i32::MAX as _;
const F32_DIFF: f32 = 0.0001;
const OPACITY_MIN: f32 = 0.0 - F32_DIFF;
const OPACITY_MAX: f32 = 1.0 + F32_DIFF;

#[inline]
unsafe fn get_slice<'a, T>(ptr: *const T, len: usize) -> Option<&'a [T]> {
    if ptr.is_null() || len * mem::size_of::<T>() > ISIZE_MAX {
        None
    } else {
        // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
        Some(slice::from_raw_parts(ptr, len))
    }
}

#[inline]
unsafe fn get_slice_check<'a, T, F>(ptr: *const T, len: usize, check: F) -> Option<&'a [T]>
where
    F: Fn((usize, &T)) -> bool,
{
    get_slice(ptr, len).and_then(|s| {
        if s.iter().enumerate().all(check) {
            Some(s)
        } else {
            None
        }
    })
}

#[inline]
unsafe fn get_mut_slice<'a, T>(ptr: *mut T, len: usize) -> Option<&'a mut [T]> {
    if ptr.is_null() || len * mem::size_of::<T>() > ISIZE_MAX {
        None
    } else {
        // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
        Some(slice::from_raw_parts_mut(ptr, len))
    }
}

fn init_model(moc: *const cubism_core_sys::csmMoc) -> Result<AlignedBytes> {
    let size = unsafe { cubism_core_sys::csmGetSizeofModel(moc) };
    if size == 0 {
        return Err(Error::InitializeModelError);
    }
    let mut model = AlignedBytes::new_zeroed(size as _, ALIGN_OF_MODEL);
    debug_assert_eq!(model.len(), size as _);

    unsafe {
        if cubism_core_sys::csmInitializeModelInPlace(moc, model.as_mut_ptr().cast(), size)
            .is_null()
        {
            Err(Error::InitializeModelError)
        } else {
            Ok(model)
        }
    }
}

#[inline]
fn convert_i32(i: i32) -> Option<usize> {
    if i >= 0 {
        Some(i as _)
    } else {
        None
    }
}

#[inline]
unsafe fn get_ids<'a>(ptr: *const *const i8, len: usize) -> Option<Box<[&'a str]>> {
    get_slice(ptr, len).and_then(|s| {
        s.iter()
            .map(|p| {
                if p.is_null() {
                    None
                } else {
                    unsafe { CStr::from_ptr(*p).to_str().ok() }
                }
            })
            .collect()
    })
}

#[inline]
fn get_ids_map<'a>(ids: &[&'a str]) -> HashMap<&'a str, usize> {
    ids.iter().enumerate().map(|(i, s)| (*s, i)).collect()
}

#[inline]
fn check_opacity(opacity: &f32) -> bool {
    (OPACITY_MIN..=OPACITY_MAX).contains(opacity)
}

#[derive(Debug)]
struct Parameters<'a> {
    ids: Box<[&'a str]>,
    ids_map: HashMap<&'a str, usize>,
    min_values: &'a [f32],
    max_values: &'a [f32],
    default_values: &'a [f32],
    values: &'a mut [f32],
    key_values: Box<[&'a [f32]]>,
}

impl<'a> Parameters<'a> {
    unsafe fn new(model: *mut cubism_core_sys::csmModel) -> Result<Self> {
        let count = convert_i32(cubism_core_sys::csmGetParameterCount(model))
            .ok_or(Error::InvalidCount("parameter"))?;
        let ids = get_ids(cubism_core_sys::csmGetParameterIds(model), count)
            .ok_or(Error::GetDataError("parameter ids"))?;
        let ids_map = get_ids_map(&ids);

        let min_values = get_slice(cubism_core_sys::csmGetParameterMinimumValues(model), count)
            .ok_or(Error::GetDataError("parameter min values"))?;

        let max_values = get_slice_check(
            cubism_core_sys::csmGetParameterMaximumValues(model),
            count,
            |(i, v)| *v >= min_values[i] - F32_DIFF,
        )
        .ok_or(Error::GetDataError("parameter max values"))?;

        let default_values = get_slice_check(
            cubism_core_sys::csmGetParameterDefaultValues(model),
            count,
            |(i, v)| (min_values[i] - F32_DIFF..=max_values[i] + F32_DIFF).contains(v),
        )
        .ok_or(Error::GetDataError("parameter default values"))?;

        let values = get_mut_slice(cubism_core_sys::csmGetParameterValues(model), count)
            .ok_or(Error::GetDataError("parameter values"))?;

        let key_values = get_slice(cubism_core_sys::csmGetParameterKeyCounts(model), count)
            .ok_or(Error::GetDataError("parameter key counts"))?
            .iter()
            .zip(
                get_slice(cubism_core_sys::csmGetParameterKeyValues(model), count)
                    .ok_or(Error::GetDataError("parameter key values"))?,
            )
            .enumerate()
            .map(|(i, (c, p))| {
                get_slice_check(*p, convert_i32(*c)?, |(_, v)| {
                    (min_values[i] - F32_DIFF..=max_values[i] + F32_DIFF).contains(v)
                })
            })
            .collect::<Option<Box<_>>>()
            .ok_or(Error::GetDataError("parameter key values"))?;

        Ok(Self {
            ids,
            ids_map,
            min_values,
            max_values,
            default_values,
            values,
            key_values,
        })
    }
}

#[derive(Debug)]
struct Parts<'a> {
    ids: Box<[&'a str]>,
    ids_map: HashMap<&'a str, usize>,
    opacities: &'a mut [f32],
    parent_indices: &'a [PartParent],
}

impl<'a> Parts<'a> {
    unsafe fn new(model: *mut cubism_core_sys::csmModel) -> Result<Self> {
        let count = convert_i32(cubism_core_sys::csmGetPartCount(model))
            .ok_or(Error::InvalidCount("part"))?;

        let ids = get_ids(cubism_core_sys::csmGetPartIds(model), count)
            .ok_or(Error::GetDataError("part ids"))?;
        let ids_map = get_ids_map(&ids);

        let opacities = get_mut_slice(cubism_core_sys::csmGetPartOpacities(model), count)
            .ok_or(Error::GetDataError("part opacities"))?;

        let parent_indices = get_slice_check(
            cubism_core_sys::csmGetPartParentPartIndices(model).cast::<PartParent>(),
            count,
            |(_, i)| i.is_valid(),
        )
        .ok_or(Error::GetDataError("part parent indices"))?;

        Ok(Self {
            ids,
            ids_map,
            opacities,
            parent_indices,
        })
    }
}

#[derive(Debug)]
struct Drawables<'a> {
    ids: Box<[&'a str]>,
    ids_map: HashMap<&'a str, usize>,
    constant_flags: &'a [ConstantFlags],
    dynamic_flags: &'a [DynamicFlags],
    texture_indices: &'a [u32],
    draw_orders: &'a [i32],
    render_orders: &'a [i32],
    opacities: &'a [f32],
    marks: Box<[&'a [u32]]>,
    vertex_positions: Box<[&'a [Vector2]]>,
    vertex_uvs: Box<[&'a [Vector2]]>,
    indices: Box<[&'a [u16]]>,
}

impl<'a> Drawables<'a> {
    unsafe fn new(model: *const cubism_core_sys::csmModel) -> Result<Self> {
        let count = convert_i32(cubism_core_sys::csmGetDrawableCount(model))
            .ok_or(Error::InvalidCount("drawable"))?;

        let ids = get_ids(cubism_core_sys::csmGetDrawableIds(model), count)
            .ok_or(Error::GetDataError("drawable ids"))?;
        let ids_map = get_ids_map(&ids);

        let constant_flags = get_slice_check(
            cubism_core_sys::csmGetDrawableConstantFlags(model).cast::<ConstantFlags>(),
            count,
            |(_, f)| f.is_valid(),
        )
        .ok_or(Error::GetDataError("drawable constant flags"))?;

        let dynamic_flags = get_slice_check(
            cubism_core_sys::csmGetDrawableDynamicFlags(model).cast::<DynamicFlags>(),
            count,
            |(_, f)| f.is_valid(),
        )
        .ok_or(Error::GetDataError("drawable dynamic flags"))?;

        let texture_indices = get_slice_check(
            cubism_core_sys::csmGetDrawableTextureIndices(model).cast::<u32>(),
            count,
            |(_, i)| *i <= I32_MAX,
        )
        .ok_or(Error::GetDataError("drawable texture indices"))?;

        let draw_orders = get_slice(cubism_core_sys::csmGetDrawableDrawOrders(model), count)
            .ok_or(Error::GetDataError("drawable draw orders"))?;

        let render_orders = get_slice(cubism_core_sys::csmGetDrawableRenderOrders(model), count)
            .ok_or(Error::GetDataError("drawable render orders"))?;

        let opacities = get_slice_check(
            cubism_core_sys::csmGetDrawableOpacities(model),
            count,
            |(_, o)| check_opacity(o),
        )
        .ok_or(Error::GetDataError("drawable opacities"))?;

        let marks = get_slice(cubism_core_sys::csmGetDrawableMaskCounts(model), count)
            .ok_or(Error::GetDataError("drawable mask counts"))?
            .iter()
            .zip(
                get_slice(
                    cubism_core_sys::csmGetDrawableMasks(model).cast::<*const u32>(),
                    count,
                )
                .ok_or(Error::GetDataError("drawable masks"))?,
            )
            .map(|(c, p)| get_slice_check(*p, convert_i32(*c)?, |(_, m)| *m <= I32_MAX))
            .collect::<Option<Box<_>>>()
            .ok_or(Error::GetDataError("drawable masks"))?;

        let vertex_counts = get_slice(cubism_core_sys::csmGetDrawableVertexCounts(model), count)
            .ok_or(Error::GetDataError("drawable vertex counts"))?;

        let vertex_positions = vertex_counts
            .iter()
            .zip(
                get_slice(
                    cubism_core_sys::csmGetDrawableVertexPositions(model).cast::<*const Vector2>(),
                    count,
                )
                .ok_or(Error::GetDataError("drawable vertex positions"))?,
            )
            .map(|(c, p)| get_slice(*p, convert_i32(*c)?))
            .collect::<Option<Box<_>>>()
            .ok_or(Error::GetDataError("drawable vertex positions"))?;

        let vertex_uvs = vertex_counts
            .iter()
            .zip(
                get_slice(
                    cubism_core_sys::csmGetDrawableVertexUvs(model).cast::<*const Vector2>(),
                    count,
                )
                .ok_or(Error::GetDataError("drawable vertex uvs"))?,
            )
            .map(|(c, p)| get_slice(*p, convert_i32(*c)?))
            .collect::<Option<Box<_>>>()
            .ok_or(Error::GetDataError("drawable vertex uvs"))?;

        let indices = get_slice(cubism_core_sys::csmGetDrawableIndexCounts(model), count)
            .ok_or(Error::GetDataError("drawable index counts"))?
            .iter()
            .zip(
                get_slice(cubism_core_sys::csmGetDrawableIndices(model), count)
                    .ok_or(Error::GetDataError("drawable indices"))?,
            )
            .map(|(c, p)| {
                // the Cubism Core doc indicate it should be 0 or a multiple of 3.
                if *c < 0 || *c % 3 != 0 {
                    Err(Error::InvalidCount("drawable indices"))
                } else {
                    get_slice(*p, *c as _).ok_or(Error::GetDataError("drawable indices"))
                }
            })
            .collect::<Result<Box<_>>>()?;

        Ok(Self {
            ids,
            ids_map,
            constant_flags,
            dynamic_flags,
            texture_indices,
            draw_orders,
            render_orders,
            opacities,
            marks,
            vertex_positions,
            vertex_uvs,
            indices,
        })
    }
}

#[derive(Debug)]
pub struct Model<'a> {
    moc: Moc,
    model: AlignedBytes,
    parameters: Parameters<'a>,
    parts: Parts<'a>,
    drawables: Drawables<'a>,
}

impl<'a> Model<'a> {
    pub fn new(moc: Moc) -> Result<Self> {
        unsafe {
            let mut model = init_model(moc.as_moc_ptr())?;
            let parameters = Parameters::new(model.as_mut_ptr().cast())?;
            let parts = Parts::new(model.as_mut_ptr().cast())?;
            let drawables = Drawables::new(model.as_ptr().cast())?;

            Ok(Self {
                moc,
                model,
                parameters,
                parts,
                drawables,
            })
        }
    }

    #[inline]
    pub fn new_from_model(model: &Self) -> Result<Self> {
        Self::new(model.moc())
    }

    #[inline]
    pub fn clone_from_model(model: &Self) -> Result<Self> {
        let mut new_model = Self::new_from_model(model)?;
        new_model.set_parameter_values(model.parameter_values());
        new_model.set_part_opacities(model.part_opacities());
        new_model.update();

        Ok(new_model)
    }

    #[inline]
    pub fn moc(&self) -> Moc {
        self.moc.clone()
    }

    #[inline]
    pub fn as_model_ptr(&self) -> *const cubism_core_sys::csmModel {
        self.model.as_ptr().cast()
    }

    #[inline]
    pub fn as_model_mut_ptr(&mut self) -> *mut cubism_core_sys::csmModel {
        self.model.as_mut_ptr().cast()
    }

    #[inline]
    pub fn update(&mut self) {
        unsafe {
            cubism_core_sys::csmResetDrawableDynamicFlags(self.as_model_mut_ptr());
            cubism_core_sys::csmUpdateModel(self.as_model_mut_ptr());
        }
    }

    pub fn read_canvas_info(&self) -> Canvas {
        let mut size_in_pixels = cubism_core_sys::csmVector2 { X: 0., Y: 0. };
        let mut origin_in_pixels = cubism_core_sys::csmVector2 { X: 0., Y: 0. };
        let mut pixels_per_unit: f32 = 0.;
        unsafe {
            cubism_core_sys::csmReadCanvasInfo(
                self.as_model_ptr(),
                &mut size_in_pixels,
                &mut origin_in_pixels,
                &mut pixels_per_unit,
            );
        }

        Canvas {
            size_in_pixels: size_in_pixels.into(),
            origin_in_pixels: origin_in_pixels.into(),
            pixels_per_unit,
        }
    }

    #[inline]
    pub fn parameter_count(&self) -> usize {
        self.parameters.ids.len()
    }

    #[inline]
    pub fn parameter_ids(&self) -> &[&str] {
        &self.parameters.ids
    }

    #[inline]
    pub fn parameter_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.parameters.ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn parameter_min_values(&self) -> &[f32] {
        self.parameters.min_values
    }

    #[inline]
    pub fn parameter_max_values(&self) -> &[f32] {
        self.parameters.max_values
    }

    #[inline]
    pub fn parameter_default_values(&self) -> &[f32] {
        self.parameters.default_values
    }

    #[inline]
    pub fn parameter_values(&self) -> &[f32] {
        self.parameters.values
    }

    #[inline]
    pub fn parameter_values_mut(&mut self) -> &mut [f32] {
        self.parameters.values
    }

    // panic
    #[inline]
    pub fn set_parameter_values<T: AsRef<[f32]>>(&mut self, values: T) {
        self.parameter_values_mut().copy_from_slice(values.as_ref());
    }

    // panic
    #[inline]
    pub fn set_parameter_value<T: AsRef<str>>(&mut self, id: T, value: f32) -> f32 {
        // SAFETY: the index from hashmap is never out of bound.
        unsafe {
            self.set_parameter_value_index_unchecked(
                self.parameter_index(id.as_ref())
                    .unwrap_or_else(|| panic!("ID {} is not exist", id.as_ref())),
                value,
            )
        }
    }

    // panic
    #[inline]
    pub fn set_parameter_value_index(&mut self, index: usize, value: f32) -> f32 {
        assert!(index < self.parameter_count());
        // SAFETY: the index has been checked.
        unsafe { self.set_parameter_value_index_unchecked(index, value) }
    }

    // safety
    #[inline]
    pub unsafe fn set_parameter_value_index_unchecked(&mut self, index: usize, value: f32) -> f32 {
        mem::replace(self.parameter_values_mut().get_unchecked_mut(index), value)
    }

    #[inline]
    pub fn parameter_key_values(&self) -> &[&[f32]] {
        &self.parameters.key_values
    }

    #[inline]
    pub fn static_parameters(&self) -> StaticParameters {
        StaticParameters::new(self)
    }

    #[inline]
    pub fn part_count(&self) -> usize {
        self.parts.ids.len()
    }

    #[inline]
    pub fn part_ids(&self) -> &[&str] {
        &self.parts.ids
    }

    #[inline]
    pub fn part_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.parts.ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn part_opacities(&self) -> &[f32] {
        self.parts.opacities
    }

    #[inline]
    pub fn part_opacities_mut(&mut self) -> &mut [f32] {
        self.parts.opacities
    }

    // panic
    #[inline]
    pub fn set_part_opacities<T: AsRef<[f32]>>(&mut self, opacities: T) {
        self.part_opacities_mut()
            .copy_from_slice(opacities.as_ref());
    }

    // panic
    #[inline]
    pub fn set_part_opacity<T: AsRef<str>>(&mut self, id: T, opacity: f32) -> f32 {
        // SAFETY: the index from hashmap is never out of bound.
        unsafe {
            self.set_part_opacity_index_unchecked(
                self.part_index(id.as_ref())
                    .unwrap_or_else(|| panic!("ID {} is not exist", id.as_ref())),
                opacity,
            )
        }
    }

    // panic
    #[inline]
    pub fn set_part_opacity_index(&mut self, index: usize, opacity: f32) -> f32 {
        assert!(index < self.part_count());
        // SAFETY: the index has been checked.
        unsafe { self.set_part_opacity_index_unchecked(index, opacity) }
    }

    // safety
    #[inline]
    pub unsafe fn set_part_opacity_index_unchecked(&mut self, index: usize, opacity: f32) -> f32 {
        mem::replace(self.part_opacities_mut().get_unchecked_mut(index), opacity)
    }

    #[inline]
    pub fn part_parent(&self) -> &[PartParent] {
        self.parts.parent_indices
    }

    #[inline]
    pub fn static_parts(&self) -> StaticParts {
        StaticParts::new(self)
    }

    #[inline]
    pub fn drawable_count(&self) -> usize {
        self.drawables.ids.len()
    }

    #[inline]
    pub fn drawable_ids(&self) -> &[&str] {
        &self.drawables.ids
    }

    #[inline]
    pub fn drawable_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.drawables.ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn drawable_constant_flags(&self) -> &[ConstantFlags] {
        self.drawables.constant_flags
    }

    #[inline]
    pub fn drawable_dynamic_flags(&self) -> Result<&[DynamicFlags]> {
        if self.drawables.dynamic_flags.iter().all(|f| f.is_valid()) {
            Ok(self.drawables.dynamic_flags)
        } else {
            Err(Error::GetDataError("drawable dynamic flags"))
        }
    }

    #[inline]
    pub fn drawable_texture_indices(&self) -> &[u32] {
        self.drawables.texture_indices
    }

    #[inline]
    pub fn drawable_draw_orders(&self) -> &[i32] {
        self.drawables.draw_orders
    }

    #[inline]
    pub fn drawable_render_orders(&self) -> &[i32] {
        self.drawables.render_orders
    }

    #[inline]
    pub fn drawable_opacities(&self) -> Result<&[f32]> {
        if self.drawables.opacities.iter().all(|o| check_opacity(o)) {
            Ok(self.drawables.opacities)
        } else {
            Err(Error::GetDataError("drawable opacities"))
        }
    }

    #[inline]
    pub fn drawable_masks(&self) -> &[&[u32]] {
        &self.drawables.marks
    }

    #[inline]
    pub fn drawable_vertex_positions(&self) -> &[&[Vector2]] {
        &self.drawables.vertex_positions
    }

    #[inline]
    pub fn drawable_vertex_uvs(&self) -> &[&[Vector2]] {
        &self.drawables.vertex_uvs
    }

    #[inline]
    pub fn drawable_indices(&self) -> &[&[u16]] {
        &self.drawables.indices
    }

    #[inline]
    pub fn static_drawables(&self) -> StaticDrawables {
        StaticDrawables::new(self)
    }

    #[inline]
    pub fn dynamic_drawables(&self) -> DynamicDrawables {
        DynamicDrawables::new(self)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Vector2(cubism_core_sys::csmVector2);

impl Vector2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self(cubism_core_sys::csmVector2 { X: x, Y: y })
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0.X
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0.Y
    }

    #[inline]
    pub fn x_y(&self) -> (f32, f32) {
        (self.0.X, self.0.Y)
    }
}

impl Default for Vector2 {
    #[inline]
    fn default() -> Self {
        Self(cubism_core_sys::csmVector2 { X: 0., Y: 0. })
    }
}

impl PartialEq for Vector2 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.X == other.0.X && self.0.Y == other.0.Y
    }
}

impl From<cubism_core_sys::csmVector2> for Vector2 {
    #[inline]
    fn from(vector: cubism_core_sys::csmVector2) -> Self {
        Self(vector)
    }
}

impl From<Vector2> for cubism_core_sys::csmVector2 {
    #[inline]
    fn from(vector: Vector2) -> Self {
        vector.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PartParent(i32);

impl PartParent {
    const ROOT: i32 = -1;

    // panic
    #[inline]
    pub fn new(parent_index: Option<usize>) -> Self {
        match parent_index {
            Some(i) => {
                assert!(i <= i32::MAX as _);
                Self(i as _)
            }
            None => Self(Self::ROOT),
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        self.0 >= Self::ROOT
    }

    #[inline]
    pub fn is_parent(&self) -> bool {
        self.0 == Self::ROOT
    }

    #[inline]
    pub fn parent(&self) -> Option<usize> {
        if self.0 <= Self::ROOT {
            None
        } else {
            Some(self.0 as _)
        }
    }
}

impl Default for PartParent {
    #[inline]
    fn default() -> Self {
        Self(Self::ROOT)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Canvas {
    pub size_in_pixels: Vector2,
    pub origin_in_pixels: Vector2,
    pub pixels_per_unit: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        log::{set_logger, DefaultLogger},
        read_haru_moc,
    };

    #[test]
    fn test_model() -> Result<()> {
        set_logger(DefaultLogger);
        let moc = read_haru_moc()?;
        let _model = Model::new(moc)?;

        Ok(())
    }
}
