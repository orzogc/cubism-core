use crate::{
    impl_iter,
    model::{Model, PartParent},
    ModelData,
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct StaticPart {
    pub index: usize,
    pub id: String,
    pub parent: PartParent,
}

#[derive(Debug)]
pub struct StaticParts<'a> {
    model: &'a Model<'a>,
    /// The initialization value is 0.
    start: usize,
    /// The initialization value is the count of parts.
    end: usize,
}

impl<'a> StaticParts<'a> {
    #[inline]
    pub(crate) fn new(model: &'a Model<'a>) -> Self {
        Self {
            model,
            start: 0,
            end: model.part_count(),
        }
    }
}

impl<'a> ModelData for StaticParts<'a> {
    type Data = StaticPart;

    #[inline]
    fn count(&self) -> usize {
        self.model.part_count()
    }

    #[inline]
    fn index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.model.part_index(id)
    }

    #[inline]
    unsafe fn get_index_unchecked(&self, index: usize) -> Self::Data {
        StaticPart {
            index,
            id: self.model.part_ids().get_unchecked(index).to_string(),
            parent: *self.model.part_parent().get_unchecked(index),
        }
    }
}

impl_iter!(StaticParts<'a>, StaticPart, Vec<StaticPart>);
