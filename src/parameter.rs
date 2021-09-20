//! Parameters of the Cubism model.

use crate::{impl_iter, Model, ModelData};

/// A static parameter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticParameter {
    /// The index of a parameter.
    pub index: usize,
    /// The ID of a parameter.
    pub id: String,
    /// The minimal value of a parameter.
    pub min_value: f32,
    /// The maximal value of a parameter.
    pub max_value: f32,
    /// The default value of a parameter.
    pub default_value: f32,
    /// The key values of a parameter.
    pub key_values: Vec<f32>,
}

/// Static parameters.
#[derive(Debug)]
pub struct StaticParameters<'a> {
    model: &'a Model<'a>,
    /// The initialization value is 0.
    start: usize,
    /// The initialization value is the count of parameters.
    end: usize,
}

impl<'a> StaticParameters<'a> {
    #[inline]
    pub(crate) fn new(model: &'a Model<'a>) -> Self {
        Self {
            model,
            start: 0,
            end: model.parameter_count(),
        }
    }
}

impl<'a> ModelData for StaticParameters<'a> {
    type Data = StaticParameter;

    #[inline]
    fn count(&self) -> usize {
        self.model.parameter_count()
    }

    #[inline]
    fn index<T: AsRef<str>>(&self, id: T) -> Option<usize> {
        self.model.parameter_index(id)
    }

    #[inline]
    unsafe fn get_index_unchecked(&self, index: usize) -> Self::Data {
        StaticParameter {
            index,
            id: self.model.parameter_ids().get_unchecked(index).to_string(),
            min_value: *self.model.parameter_min_values().get_unchecked(index),
            max_value: *self.model.parameter_max_values().get_unchecked(index),
            default_value: *self.model.parameter_default_values().get_unchecked(index),
            key_values: self
                .model
                .parameter_key_values()
                .get_unchecked(index)
                .to_vec(),
        }
    }
}

impl_iter!(StaticParameters<'a>, StaticParameter, Vec<StaticParameter>);
