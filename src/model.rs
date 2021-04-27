use crate::error::{Error, Result};
use crate::flags::{ConstantFlags, DynamicFlags};
use crate::moc::Moc;
use crate::ALIGN_OF_MODEL;
use aligned_utils::bytes::AlignedBytes;
use core::ptr::NonNull;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use std::collections::HashMap;
use std::ffi::CStr;

#[inline]
fn get_slice<'a, T>(ptr: *const T, len: usize) -> Option<&'a [T]> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
        Some(unsafe { from_raw_parts(ptr, len) })
    }
}

#[inline]
fn get_mut_slice<'a, T>(ptr: *mut T, len: usize) -> Option<&'a mut [T]> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
        Some(unsafe { from_raw_parts_mut(ptr, len) })
    }
}

fn init_model(moc: *const cubism_core_sys::csmMoc) -> Result<AlignedBytes> {
    let size = unsafe { cubism_core_sys::csmGetSizeofModel(moc) };
    let mut model = AlignedBytes::new_zeroed(size as _, ALIGN_OF_MODEL);
    if unsafe { cubism_core_sys::csmInitializeModelInPlace(moc, model.as_mut_ptr() as _, size) }
        .is_null()
    {
        Err(Error::InitializeModelError)
    } else {
        Ok(model)
    }
}

#[derive(Clone, Debug)]
struct StaticData {
    /// the ID of the parameter
    parameter_ids: Vec<String>,
    parameter_ids_map: HashMap<String, usize>,
    parameter_min_values: Vec<f32>,
    parameter_max_values: Vec<f32>,
    parameter_default_values: Vec<f32>,
    part_ids: Vec<String>,
    part_ids_map: HashMap<String, usize>,
    part_parent_indices: Vec<PartParent>,
    drawable_ids: Vec<String>,
    drawable_ids_map: HashMap<String, usize>,
    drawable_constant_flags: Vec<ConstantFlags>,
    drawable_texture_indices: Vec<i32>,
    drawable_marks: Vec<Vec<i32>>,
    drawable_vertex_counts: Vec<usize>,
    drawable_vertex_uvs: Vec<Vec<Vector2>>,
    drawable_indices: Vec<Vec<u16>>,
}

impl StaticData {
    fn new(model: *const cubism_core_sys::csmModel) -> Result<Self> {
        let check_count = |i| if i < 0 { None } else { Some(i as usize) };
        let get_ids = |ptr: *const *const i8, len| -> Option<Vec<_>> {
            if ptr.is_null() {
                None
            } else {
                // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
                unsafe { from_raw_parts(ptr, len) }
                    .iter()
                    .map(|&ptr| {
                        if ptr.is_null() {
                            None
                        } else {
                            Some(
                                // SAFETY: it's safe here because the pointer points to a C string.
                                unsafe { CStr::from_ptr(ptr) }
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                        }
                    })
                    .collect()
            }
        };
        let get_ids_map = |ids: &[String]| {
            ids.iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), i))
                .collect()
        };

        let parameter_count = check_count(unsafe { cubism_core_sys::csmGetParameterCount(model) })
            .ok_or(Error::InvalidDataCount("parameter"))?;
        let part_count = check_count(unsafe { cubism_core_sys::csmGetPartCount(model) })
            .ok_or(Error::InvalidDataCount("part"))?;
        let drawable_count = check_count(unsafe { cubism_core_sys::csmGetDrawableCount(model) })
            .ok_or(Error::InvalidDataCount("drawable"))?;

        let parameter_ids = get_ids(
            unsafe { cubism_core_sys::csmGetParameterIds(model) },
            parameter_count,
        )
        .ok_or(Error::GetDataError("parameter ids"))?;
        let parameter_ids_map = get_ids_map(parameter_ids.as_slice());

        let parameter_min_values = get_slice(
            unsafe { cubism_core_sys::csmGetParameterMinimumValues(model) },
            parameter_count,
        )
        .ok_or(Error::GetDataError("parameter min values"))?
        .to_vec();

        let parameter_max_values = get_slice(
            unsafe { cubism_core_sys::csmGetParameterMaximumValues(model) },
            parameter_count,
        )
        .ok_or(Error::GetDataError("parameter max values"))?
        .to_vec();

        let parameter_default_values = get_slice(
            unsafe { cubism_core_sys::csmGetParameterDefaultValues(model) },
            parameter_count,
        )
        .ok_or(Error::GetDataError("parameter default values"))?
        .to_vec();

        let part_ids = get_ids(unsafe { cubism_core_sys::csmGetPartIds(model) }, part_count)
            .ok_or(Error::GetDataError("part ids"))?;
        let part_ids_map = get_ids_map(part_ids.as_slice());

        let part_parent_indices = get_slice(
            unsafe { cubism_core_sys::csmGetPartParentPartIndices(model) },
            part_count,
        )
        .ok_or(Error::GetDataError("part parent indices"))?
        .iter()
        .map(|&i| PartParent::new(i))
        .collect::<Result<_>>()?;

        let drawable_ids = get_ids(
            unsafe { cubism_core_sys::csmGetDrawableIds(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable ids"))?;
        let drawable_ids_map = get_ids_map(drawable_ids.as_slice());

        let drawable_constant_flags = get_slice(
            unsafe { cubism_core_sys::csmGetDrawableConstantFlags(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable constant flags"))?
        .iter()
        .map(|&f| ConstantFlags::from_bits(f).ok_or(Error::InvalidFlags("constant", f)))
        .collect::<Result<_>>()?;

        let drawable_texture_indices = get_slice(
            unsafe { cubism_core_sys::csmGetDrawableTextureIndices(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable texture indices"))?
        .to_vec();

        let drawable_marks = get_slice(
            unsafe { cubism_core_sys::csmGetDrawableMaskCounts(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable mask counts"))?
        .iter()
        .zip(
            get_slice(
                unsafe { cubism_core_sys::csmGetDrawableMasks(model) },
                drawable_count,
            )
            .ok_or(Error::GetDataError("drawable masks"))?,
        )
        .map(|(&c, &p)| {
            if c < 0 {
                Err(Error::InvalidDataCount("drawable mask"))
            } else if p.is_null() {
                Err(Error::GetDataError("drawable masks because null pointer"))
            } else {
                // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
                Ok(unsafe { from_raw_parts(p, c as _) }.to_vec())
            }
        })
        .collect::<Result<_>>()?;

        let drawable_vertex_counts = get_slice(
            unsafe { cubism_core_sys::csmGetDrawableVertexCounts(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable vertex counts"))?
        .iter()
        .map(|&c| check_count(c))
        .collect::<Option<Vec<_>>>()
        .ok_or(Error::InvalidDataCount("drawable vertex"))?;

        let drawable_vertex_uvs = drawable_vertex_counts
            .iter()
            .zip(
                get_slice(
                    unsafe { cubism_core_sys::csmGetDrawableVertexUvs(model) },
                    drawable_count,
                )
                .ok_or(Error::GetDataError("drawable vertex uvs"))?,
            )
            .map(|(&c, &p)| {
                if p.is_null() {
                    Err(Error::GetDataError(
                        "drawable vertex uvs because null pointer",
                    ))
                } else {
                    // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
                    Ok(unsafe { from_raw_parts(p, c) }
                        .iter()
                        .map(|&v| Vector2::new(v))
                        .collect())
                }
            })
            .collect::<Result<_>>()?;

        let drawable_indices = get_slice(
            unsafe { cubism_core_sys::csmGetDrawableIndexCounts(model) },
            drawable_count,
        )
        .ok_or(Error::GetDataError("drawable index counts"))?
        .iter()
        .zip(
            get_slice(
                unsafe { cubism_core_sys::csmGetDrawableIndices(model) },
                drawable_count,
            )
            .ok_or(Error::GetDataError("drawable indices"))?,
        )
        .map(|(&c, &p)| {
            // the official doc say it should be 0 or a multiple of 3.
            if c < 0 || c % 3 != 0 {
                Err(Error::InvalidDataCount("drawable indices"))
            } else if p.is_null() {
                Err(Error::GetDataError("drawable indices because null pointer"))
            } else {
                // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
                Ok(unsafe { from_raw_parts(p, c as _) }.to_vec())
            }
        })
        .collect::<Result<_>>()?;

        Ok(Self {
            parameter_ids,
            parameter_ids_map,
            parameter_min_values,
            parameter_max_values,
            parameter_default_values,
            part_ids,
            part_ids_map,
            part_parent_indices,
            drawable_ids,
            drawable_ids_map,
            drawable_constant_flags,
            drawable_texture_indices,
            drawable_marks,
            drawable_vertex_counts,
            drawable_vertex_uvs,
            drawable_indices,
        })
    }
}

#[derive(Debug)]
struct MutableData {
    parameter_values: NonNull<[f32]>,
    part_opacities: NonNull<[f32]>,
}

impl MutableData {
    fn new(
        model: *mut cubism_core_sys::csmModel,
        parameter_count: usize,
        part_count: usize,
    ) -> Result<Self> {
        let parameter_values = NonNull::from(
            get_mut_slice(
                unsafe { cubism_core_sys::csmGetParameterValues(model) },
                parameter_count,
            )
            .ok_or(Error::GetDataError("parameter values"))?,
        );
        let part_opacities = NonNull::from(
            get_mut_slice(
                unsafe { cubism_core_sys::csmGetPartOpacities(model) },
                part_count,
            )
            .ok_or(Error::GetDataError("part opacities"))?,
        );

        Ok(Self {
            parameter_values,
            part_opacities,
        })
    }
}

unsafe impl Send for MutableData {}
unsafe impl Sync for MutableData {}

/// Cubism model.
#[derive(Debug)]
pub struct Model {
    model: AlignedBytes,
    moc: Moc,
    static_data: StaticData,
    mut_data: MutableData,
}

impl Model {
    pub fn new(moc: Moc) -> Result<Self> {
        let mut model = init_model(moc.as_moc_ptr())?;
        let static_data = StaticData::new(model.as_ptr() as _)?;
        let mut_data = MutableData::new(
            model.as_mut_ptr() as _,
            static_data.parameter_ids.len(),
            static_data.part_ids.len(),
        )?;

        Ok(Self {
            model,
            moc,
            static_data,
            mut_data,
        })
    }

    #[inline]
    pub fn from_model(model: Self) -> Result<Self> {
        Self::new(model.moc())
    }

    #[inline]
    pub fn moc(&self) -> Moc {
        self.moc.clone()
    }

    #[inline]
    fn as_model_ptr(&self) -> *const cubism_core_sys::csmModel {
        self.model.as_ptr() as _
    }

    #[inline]
    fn as_model_mut_ptr(&mut self) -> *mut cubism_core_sys::csmModel {
        self.model.as_mut_ptr() as _
    }

    #[inline]
    pub fn parameter_count(&self) -> usize {
        self.static_data.parameter_ids.len()
    }

    #[inline]
    pub fn parameter_ids(&self) -> &[String] {
        self.static_data.parameter_ids.as_slice()
    }

    #[inline]
    pub fn get_parameter_id_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.static_data.parameter_ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn parameter_min_values(&self) -> &[f32] {
        self.static_data.parameter_min_values.as_slice()
    }

    #[inline]
    pub fn parameter_max_values(&self) -> &[f32] {
        self.static_data.parameter_max_values.as_slice()
    }

    #[inline]
    pub fn parameter_default_values(&self) -> &[f32] {
        self.static_data.parameter_default_values.as_slice()
    }

    #[inline]
    pub fn parameter_values(&self) -> &[f32] {
        // SAFETY: it's safe as long as `self` is immutable.
        unsafe { self.mut_data.parameter_values.as_ref() }
    }

    #[inline]
    pub fn parameter_values_mut(&mut self) -> &mut [f32] {
        // SAFETY: it's safe as long as `self` is mutable.
        unsafe { self.mut_data.parameter_values.as_mut() }
    }

    #[inline]
    pub fn set_parameter_values<T: AsRef<[f32]>>(&mut self, values: T) {
        self.parameter_values_mut().copy_from_slice(values.as_ref());
    }

    /// If it fails, it returns `None`, otherwise returns the old value.
    #[inline]
    pub fn set_parameter_value<T: AsRef<str>>(&mut self, id: T, value: f32) -> Option<f32> {
        let index = self.get_parameter_id_index(id)?;
        // SAFETY: the index from hashmap is never out of bound.
        Some(unsafe { self.set_parameter_value_index_unchecked(index, value) })
    }

    /// If it fails, it returns `None`, otherwise returns the old value.
    #[inline]
    pub fn set_parameter_value_index(&mut self, index: usize, value: f32) -> Option<f32> {
        if index < self.parameter_count() {
            // SAFETY: index has been checked.
            Some(unsafe { self.set_parameter_value_index_unchecked(index, value) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn set_parameter_value_index_unchecked(&mut self, index: usize, value: f32) -> f32 {
        //let value = value
        //    .max(*self.parameter_min_values().get_unchecked(index))
        //    .min(*self.parameter_max_values().get_unchecked(index));
        core::mem::replace(self.parameter_values_mut().get_unchecked_mut(index), value)
    }

    #[inline]
    pub fn parameter_value_index(&mut self, index: usize, value: f32) -> f32 {
        assert!(index < self.parameter_count());
        // SAFETY: index has been checked.
        unsafe { self.set_parameter_value_index_unchecked(index, value) }
    }

    #[inline]
    pub fn get_static_parameter(&self, index: usize) -> Option<StaticParameter> {
        if index < self.parameter_count() {
            // SAFETY: index has been checked.
            Some(unsafe { self.get_static_parameter_unchecked(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_static_parameter_unchecked(&self, index: usize) -> StaticParameter {
        StaticParameter {
            index,
            id: self.parameter_ids().get_unchecked(index).clone(),
            min_value: *self.parameter_min_values().get_unchecked(index),
            max_value: *self.parameter_max_values().get_unchecked(index),
            default_value: *self.parameter_default_values().get_unchecked(index),
        }
    }

    #[inline]
    pub fn static_parameter<T: AsRef<str>>(&self, id: T) -> Option<StaticParameter> {
        // SAFETY: the index from hashmap is never out of bound.
        self.get_parameter_id_index(id)
            .map(|i| unsafe { self.get_static_parameter_unchecked(i) })
    }

    #[inline]
    pub fn static_parameter_index(&self, index: usize) -> StaticParameter {
        assert!(index < self.parameter_count());
        // SAFETY: index has been checked.
        unsafe { self.get_static_parameter_unchecked(index) }
    }

    #[inline]
    pub fn static_parameter_vec(&self) -> Vec<StaticParameter> {
        self.static_parameter_iter().collect()
    }

    #[inline]
    pub fn static_parameter_iter(&self) -> StaticParameterIter {
        StaticParameterIter {
            model: self,
            len: self.parameter_count(),
            index: 0,
        }
    }

    #[inline]
    pub fn part_count(&self) -> usize {
        self.static_data.part_ids.len()
    }

    #[inline]
    pub fn part_ids(&self) -> &[String] {
        self.static_data.part_ids.as_slice()
    }

    #[inline]
    pub fn get_part_id_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.static_data.part_ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn part_opacities(&self) -> &[f32] {
        // SAFETY: it's safe as long as `self` is immutable.
        unsafe { self.mut_data.part_opacities.as_ref() }
    }

    #[inline]
    pub fn part_opacities_mut(&mut self) -> &mut [f32] {
        // SAFETY: it's safe as long as `self` is mutable.
        unsafe { self.mut_data.part_opacities.as_mut() }
    }

    #[inline]
    pub fn set_part_opacities<T: AsRef<[f32]>>(&mut self, values: T) {
        self.part_opacities_mut().copy_from_slice(values.as_ref());
    }

    /// If it fails, it returns `None`, otherwise returns the old value.
    #[inline]
    pub fn set_part_opacity<T: AsRef<str>>(&mut self, id: T, value: f32) -> Option<f32> {
        let index = self.get_part_id_index(id)?;
        // SAFETY: the index from hashmap is never out of bound.
        Some(unsafe { self.set_part_opacity_index_unchecked(index, value) })
    }

    /// If it fails, it returns `None`, otherwise returns the old value.
    #[inline]
    pub fn set_part_opacity_index(&mut self, index: usize, value: f32) -> Option<f32> {
        if index < self.part_count() {
            // SAFETY: index has been checked.
            Some(unsafe { self.set_part_opacity_index_unchecked(index, value) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn set_part_opacity_index_unchecked(&mut self, index: usize, value: f32) -> f32 {
        //let value = value.max(0.).min(1.);
        core::mem::replace(self.part_opacities_mut().get_unchecked_mut(index), value)
    }

    #[inline]
    pub fn part_opacity_index(&mut self, index: usize, value: f32) -> f32 {
        assert!(index < self.part_count());
        // SAFETY: index has been checked.
        unsafe { self.set_part_opacity_index_unchecked(index, value) }
    }

    #[inline]
    pub fn part_parent_indices(&self) -> &[PartParent] {
        self.static_data.part_parent_indices.as_slice()
    }

    #[inline]
    pub fn get_static_part(&self, index: usize) -> Option<StaticPart> {
        if index < self.part_count() {
            // SAFETY: index has been checked.
            Some(unsafe { self.get_static_part_unchecked(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_static_part_unchecked(&self, index: usize) -> StaticPart {
        StaticPart {
            index,
            id: self.part_ids().get_unchecked(index).clone(),
            part_parent_index: *self.part_parent_indices().get_unchecked(index),
        }
    }

    #[inline]
    pub fn static_part<T: AsRef<str>>(&self, id: T) -> Option<StaticPart> {
        // SAFETY: the index from hashmap is never out of bound.
        self.get_part_id_index(id)
            .map(|i| unsafe { self.get_static_part_unchecked(i) })
    }

    #[inline]
    pub fn static_part_index(&self, index: usize) -> StaticPart {
        assert!(index < self.part_count());
        // SAFETY: index has been checked.
        unsafe { self.get_static_part_unchecked(index) }
    }

    #[inline]
    pub fn static_part_vec(&self) -> Vec<StaticPart> {
        self.static_part_iter().collect()
    }

    #[inline]
    pub fn static_part_iter(&self) -> StaticPartIter {
        StaticPartIter {
            model: self,
            len: self.part_count(),
            index: 0,
        }
    }

    #[inline]
    pub fn drawable_count(&self) -> usize {
        self.static_data.drawable_ids.len()
    }

    #[inline]
    pub fn drawable_ids(&self) -> &[String] {
        self.static_data.drawable_ids.as_slice()
    }

    #[inline]
    pub fn get_drawable_id_index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.static_data.drawable_ids_map.get(id.as_ref()).copied()
    }

    #[inline]
    pub fn drawable_constant_flags(&self) -> &[ConstantFlags] {
        self.static_data.drawable_constant_flags.as_slice()
    }

    pub fn drawable_dynamic_flags(&self) -> Result<Vec<DynamicFlags>> {
        get_slice(
            unsafe { cubism_core_sys::csmGetDrawableDynamicFlags(self.as_model_ptr()) },
            self.drawable_count(),
        )
        .ok_or(Error::GetDataError("drawable dynamic flags"))?
        .iter()
        .map(|&f| DynamicFlags::from_bits(f).ok_or(Error::InvalidFlags("dynamic", f)))
        .collect()
    }

    #[inline]
    pub fn drawable_texture_indices(&self) -> &[i32] {
        self.static_data.drawable_texture_indices.as_slice()
    }

    #[inline]
    pub fn drawable_draw_orders(&self) -> Result<&[i32]> {
        get_slice(
            unsafe { cubism_core_sys::csmGetDrawableDrawOrders(self.as_model_ptr()) },
            self.drawable_count(),
        )
        .ok_or(Error::GetDataError("drawable draw orders"))
    }

    #[inline]
    pub fn drawable_render_orders(&self) -> Result<&[i32]> {
        get_slice(
            unsafe { cubism_core_sys::csmGetDrawableRenderOrders(self.as_model_ptr()) },
            self.drawable_count(),
        )
        .ok_or(Error::GetDataError("drawable render orders"))
    }

    #[inline]
    pub fn drawable_opacities(&self) -> Result<&[f32]> {
        get_slice(
            unsafe { cubism_core_sys::csmGetDrawableOpacities(self.as_model_ptr()) },
            self.drawable_count(),
        )
        .ok_or(Error::GetDataError("drawable opacities"))
    }

    #[inline]
    pub fn drawable_masks(&self) -> &[Vec<i32>] {
        self.static_data.drawable_marks.as_slice()
    }

    pub fn drawable_vertex_positions(&self) -> Result<Vec<Vec<Vector2>>> {
        self.static_data
            .drawable_vertex_counts
            .iter()
            .zip(
                get_slice(
                    unsafe { cubism_core_sys::csmGetDrawableVertexPositions(self.as_model_ptr()) },
                    self.drawable_count(),
                )
                .ok_or(Error::GetDataError("drawable vertex positions"))?,
            )
            .map(|(&c, &p)| {
                if p.is_null() {
                    Err(Error::GetDataError(
                        "drawable vertex positions because null pointer",
                    ))
                } else {
                    // SAFETY: it's safe here because the memory of a C/C++ array is contiguous.
                    Ok(unsafe { from_raw_parts(p, c) }
                        .iter()
                        .map(|&v| Vector2::new(v))
                        .collect())
                }
            })
            .collect()
    }

    #[inline]
    pub fn drawable_vertex_uvs(&self) -> &[Vec<Vector2>] {
        self.static_data.drawable_vertex_uvs.as_slice()
    }

    #[inline]
    pub fn drawable_indices(&self) -> &[Vec<u16>] {
        self.static_data.drawable_indices.as_slice()
    }

    #[inline]
    pub fn get_static_drawable(&self, index: usize) -> Option<StaticDrawable> {
        if index < self.drawable_count() {
            // SAFETY: index has been checked.
            Some(unsafe { self.get_static_drawable_unchecked(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_static_drawable_unchecked(&self, index: usize) -> StaticDrawable {
        StaticDrawable {
            index,
            id: self.drawable_ids().get_unchecked(index).clone(),
            constant_flag: *self.drawable_constant_flags().get_unchecked(index),
            texture_index: *self.drawable_texture_indices().get_unchecked(index),
            masks: self.drawable_masks().get_unchecked(index).clone(),
            vertex_uvs: self.drawable_vertex_uvs().get_unchecked(index).clone(),
            indices: self.drawable_indices().get_unchecked(index).clone(),
        }
    }

    #[inline]
    pub fn static_drawable<T: AsRef<str>>(&self, id: T) -> Option<StaticDrawable> {
        // SAFETY: the index from hashmap is never out of bound.
        self.get_drawable_id_index(id)
            .map(|i| unsafe { self.get_static_drawable_unchecked(i) })
    }

    #[inline]
    pub fn static_drawable_index(&self, index: usize) -> StaticDrawable {
        assert!(index < self.drawable_count());
        // SAFETY: index has been checked.
        unsafe { self.get_static_drawable_unchecked(index) }
    }

    #[inline]
    pub fn static_drawable_vec(&self) -> Vec<StaticDrawable> {
        self.static_drawable_iter().collect()
    }

    #[inline]
    pub fn static_drawable_iter(&self) -> StaticDrawableIter {
        StaticDrawableIter {
            model: self,
            len: self.drawable_count(),
            index: 0,
        }
    }

    #[inline]
    pub fn get_dynamic_drawable(&self, index: usize) -> Result<DynamicDrawable> {
        if index < self.drawable_count() {
            // SAFETY: index has been checked.
            unsafe { self.get_dynamic_drawable_unchecked(index) }
        } else {
            Err(Error::InvalidDataCount("drawable dynamic"))
        }
    }

    #[inline]
    pub unsafe fn get_dynamic_drawable_unchecked(&self, index: usize) -> Result<DynamicDrawable> {
        Ok(DynamicDrawable {
            index,
            id: self.drawable_ids().get_unchecked(index).clone(),
            dynamic_flag: *self.drawable_dynamic_flags()?.get_unchecked(index),
            draw_order: *self.drawable_draw_orders()?.get_unchecked(index),
            render_order: *self.drawable_render_orders()?.get_unchecked(index),
            opacity: *self.drawable_opacities()?.get_unchecked(index),
            vertex_positions: self
                .drawable_vertex_positions()?
                .get_unchecked(index)
                .clone(),
        })
    }

    #[inline]
    pub fn dynamic_drawable<T: AsRef<str>>(&self, id: T) -> Result<DynamicDrawable> {
        // SAFETY: the index from hashmap is never out of bound.
        self.get_drawable_id_index(id)
            .ok_or(Error::InvalidDataCount("drawable dynamic"))
            .and_then(|i| unsafe { self.get_dynamic_drawable_unchecked(i) })
    }

    #[inline]
    pub fn dynamic_drawable_index(&self, index: usize) -> Result<DynamicDrawable> {
        assert!(index < self.drawable_count());
        // SAFETY: index has been checked.
        unsafe { self.get_dynamic_drawable_unchecked(index) }
    }

    #[inline]
    pub fn dynamic_drawable_vec(&self) -> Result<Vec<DynamicDrawable>> {
        self.dynamic_drawable_iter().collect()
    }

    #[inline]
    pub fn dynamic_drawable_iter(&self) -> DynamicDrawableIter {
        DynamicDrawableIter {
            model: self,
            len: self.drawable_count(),
            index: 0,
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
    pub fn update_model(&mut self) {
        unsafe {
            cubism_core_sys::csmResetDrawableDynamicFlags(self.as_model_mut_ptr());
            cubism_core_sys::csmUpdateModel(self.as_model_mut_ptr());
        }
    }
}

impl Clone for Model {
    #[inline]
    fn clone(&self) -> Self {
        let moc = self.moc();
        let mut model =
            init_model(moc.as_moc_ptr()).expect("failed to init model when cloning `Model`"); // it should not fail.
        let static_data = self.static_data.clone();
        let mut_data = MutableData::new(
            model.as_mut_ptr() as _,
            static_data.parameter_ids.len(),
            static_data.part_ids.len(),
        )
        .expect("failed to new mutable data when cloning `Model`"); // it should not fail.

        let mut model = Self {
            model,
            moc,
            static_data,
            mut_data,
        };
        model.set_parameter_values(self.parameter_values());
        model.set_part_opacities(self.part_opacities());

        model
    }
}

impl core::convert::TryFrom<Moc> for Model {
    type Error = Error;

    #[inline]
    fn try_from(moc: Moc) -> Result<Self> {
        Self::new(moc)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticParameter {
    pub index: usize,
    pub id: String,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
}

#[derive(Clone, Debug)]
pub struct StaticParameterIter<'a> {
    model: &'a Model,
    len: usize,
    index: usize,
}

impl<'a> Iterator for StaticParameterIter<'a> {
    type Item = StaticParameter;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: index has been checked.
            let parameter = unsafe { self.model.get_static_parameter_unchecked(self.index) };
            self.index += 1;
            Some(parameter)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len - self.index;
        (remain, Some(remain))
    }
}

impl<'a> DoubleEndedIterator for StaticParameterIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: it's never out of bound.
            let parameter = unsafe { self.model.get_static_parameter_unchecked(self.len - 1) };
            self.len -= 1;
            Some(parameter)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for StaticParameterIter<'a> {}
impl<'a> core::iter::FusedIterator for StaticParameterIter<'a> {}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticPart {
    pub index: usize,
    pub id: String,
    pub part_parent_index: PartParent,
}

#[derive(Clone, Debug)]
pub struct StaticPartIter<'a> {
    model: &'a Model,
    len: usize,
    index: usize,
}

impl<'a> Iterator for StaticPartIter<'a> {
    type Item = StaticPart;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: index has been checked.
            let part = unsafe { self.model.get_static_part_unchecked(self.index) };
            self.index += 1;
            Some(part)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len - self.index;
        (remain, Some(remain))
    }
}

impl<'a> DoubleEndedIterator for StaticPartIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: it's never out of bound.
            let part = unsafe { self.model.get_static_part_unchecked(self.len - 1) };
            self.len -= 1;
            Some(part)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for StaticPartIter<'a> {}
impl<'a> core::iter::FusedIterator for StaticPartIter<'a> {}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticDrawable {
    pub index: usize,
    pub id: String,
    pub constant_flag: ConstantFlags,
    pub texture_index: i32,
    pub masks: Vec<i32>,
    pub vertex_uvs: Vec<Vector2>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct StaticDrawableIter<'a> {
    model: &'a Model,
    len: usize,
    index: usize,
}

impl<'a> Iterator for StaticDrawableIter<'a> {
    type Item = StaticDrawable;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: index has been checked.
            let drawable_static = unsafe { self.model.get_static_drawable_unchecked(self.index) };
            self.index += 1;
            Some(drawable_static)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len - self.index;
        (remain, Some(remain))
    }
}

impl<'a> DoubleEndedIterator for StaticDrawableIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: it's never out of bound.
            let drawable_static = unsafe { self.model.get_static_drawable_unchecked(self.len - 1) };
            self.len -= 1;
            Some(drawable_static)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for StaticDrawableIter<'a> {}
impl<'a> core::iter::FusedIterator for StaticDrawableIter<'a> {}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicDrawable {
    pub index: usize,
    pub id: String,
    pub dynamic_flag: DynamicFlags,
    pub draw_order: i32,
    pub render_order: i32,
    pub opacity: f32,
    pub vertex_positions: Vec<Vector2>,
}

#[derive(Clone, Debug)]
pub struct DynamicDrawableIter<'a> {
    model: &'a Model,
    len: usize,
    index: usize,
}

impl<'a> Iterator for DynamicDrawableIter<'a> {
    type Item = Result<DynamicDrawable>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: index has been checked.
            let drawable_dynamic = unsafe { self.model.get_dynamic_drawable_unchecked(self.index) };
            self.index += 1;
            Some(drawable_dynamic)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len - self.index;
        (remain, Some(remain))
    }
}

impl<'a> DoubleEndedIterator for DynamicDrawableIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            // SAFETY: it's never out of bound.
            let drawable_dynamic =
                unsafe { self.model.get_dynamic_drawable_unchecked(self.len - 1) };
            self.len -= 1;
            Some(drawable_dynamic)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for DynamicDrawableIter<'a> {}
impl<'a> core::iter::FusedIterator for DynamicDrawableIter<'a> {}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    #[inline]
    pub fn new(vertex: cubism_core_sys::csmVector2) -> Self {
        Self {
            x: vertex.X,
            y: vertex.Y,
        }
    }
}

impl From<cubism_core_sys::csmVector2> for Vector2 {
    #[inline]
    fn from(vertex: cubism_core_sys::csmVector2) -> Self {
        Self::new(vertex)
    }
}

impl From<[f32; 2]> for Vector2 {
    #[inline]
    fn from(vertex: [f32; 2]) -> Self {
        Self {
            x: vertex[0],
            y: vertex[1],
        }
    }
}

impl From<Vector2> for [f32; 2] {
    #[inline]
    fn from(vertex: Vector2) -> Self {
        [vertex.x, vertex.y]
    }
}

impl From<Vector2> for Vec<f32> {
    #[inline]
    fn from(vertex: Vector2) -> Self {
        vec![vertex.x, vertex.y]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PartParent {
    Root,
    Parent(usize),
}

impl PartParent {
    #[inline]
    pub fn new(index: i32) -> Result<Self> {
        match index {
            i if i < -1 => Err(Error::InvalidDataCount("part parent")),
            -1 => Ok(Self::Root),
            _ => Ok(Self::Parent(index as usize)),
        }
    }

    /// Returns `true` if the part_parent is [`Root`](PartParent::Root).
    #[inline]
    pub fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    /// Returns `true` if the part_parent is [`Parent`](PartParent::Parent).
    #[inline]
    pub fn is_parent(&self) -> bool {
        matches!(self, Self::Parent(..))
    }

    #[inline]
    pub fn parent_index(&self) -> Option<usize> {
        match self {
            PartParent::Root => None,
            PartParent::Parent(i) => Some(*i),
        }
    }
}

impl core::convert::TryFrom<i32> for PartParent {
    type Error = Error;

    #[inline]
    fn try_from(index: i32) -> Result<Self> {
        Self::new(index)
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
    use crate::read_haru_moc;

    #[test]
    fn test_model() -> Result<()> {
        let moc = read_haru_moc()?;
        let _model = Model::new(moc)?;
        //println!("{:?}", model.parameters());
        //println!("{:?}", model.parts());
        //println!("{:?}", model.drawable_statics());
        //println!("{:?}", model.drawable_dynamics());

        Ok(())
    }
}
