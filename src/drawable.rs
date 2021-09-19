use crate::{
    impl_iter,
    model::{Model, Vector2},
    ConstantFlags, DynamicFlags, ModelData, Result,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StaticDrawable {
    pub index: usize,
    pub id: String,
    pub constant_flags: ConstantFlags,
    pub texture_index: usize,
    pub masks: Vec<usize>,
    pub vertex_uvs: Vec<Vector2>,
    pub indices: Vec<usize>,
}

#[derive(Debug)]
pub struct StaticDrawables<'a> {
    model: &'a Model<'a>,
    /// The initialization value is 0.
    start: usize,
    /// The initialization value is the count of drawables.
    end: usize,
}

impl<'a> StaticDrawables<'a> {
    #[inline]
    pub(crate) fn new(model: &'a Model<'a>) -> Self {
        Self {
            model,
            start: 0,
            end: model.drawable_count(),
        }
    }
}

impl<'a> ModelData for StaticDrawables<'a> {
    type Data = StaticDrawable;

    #[inline]
    fn count(&self) -> usize {
        self.model.drawable_count()
    }

    #[inline]
    fn index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.model.drawable_index(id)
    }

    #[inline]
    unsafe fn get_index_unchecked(&self, index: usize) -> Self::Data {
        StaticDrawable {
            index,
            id: self.model.drawable_ids().get_unchecked(index).to_string(),
            constant_flags: *self.model.drawable_constant_flags().get_unchecked(index),
            texture_index: *self.model.drawable_texture_indices().get_unchecked(index) as _,
            masks: self
                .model
                .drawable_masks()
                .get_unchecked(index)
                .iter()
                .map(|m| *m as usize)
                .collect(),
            vertex_uvs: self
                .model
                .drawable_vertex_uvs()
                .get_unchecked(index)
                .to_vec(),
            indices: self
                .model
                .drawable_indices()
                .get_unchecked(index)
                .iter()
                .map(|i| *i as usize)
                .collect(),
        }
    }
}

impl_iter!(StaticDrawables<'a>, StaticDrawable, Vec<StaticDrawable>);

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicDrawable {
    pub index: usize,
    pub id: String,
    pub dynamic_flags: DynamicFlags,
    pub draw_order: i32,
    pub render_order: i32,
    pub opacity: f32,
    pub vertex_positions: Vec<Vector2>,
}

#[derive(Debug)]
pub struct DynamicDrawables<'a> {
    model: &'a Model<'a>,
    /// The initialization value is 0.
    start: usize,
    /// The initialization value is the count of drawables.
    end: usize,
}

impl<'a> DynamicDrawables<'a> {
    #[inline]
    pub(crate) fn new(model: &'a Model<'a>) -> Self {
        Self {
            model,
            start: 0,
            end: model.drawable_count(),
        }
    }
}

impl<'a> ModelData for DynamicDrawables<'a> {
    type Data = Result<DynamicDrawable>;

    #[inline]
    fn count(&self) -> usize {
        self.model.drawable_count()
    }

    #[inline]
    fn index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.model.drawable_index(id)
    }

    #[inline]
    unsafe fn get_index_unchecked(&self, index: usize) -> Self::Data {
        Ok(DynamicDrawable {
            index,
            id: self.model.drawable_ids().get_unchecked(index).to_string(),
            dynamic_flags: *self.model.drawable_dynamic_flags()?.get_unchecked(index),
            draw_order: *self.model.drawable_draw_orders().get_unchecked(index),
            render_order: *self.model.drawable_render_orders().get_unchecked(index),
            opacity: *self.model.drawable_opacities()?.get_unchecked(index),
            vertex_positions: self
                .model
                .drawable_vertex_positions()
                .get_unchecked(index)
                .to_vec(),
        })
    }
}

impl_iter!(
    DynamicDrawables<'a>,
    Result<DynamicDrawable>,
    Result<Vec<DynamicDrawable>>
);
