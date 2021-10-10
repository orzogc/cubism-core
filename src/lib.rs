//! Rust API for [Cubism Core native library](https://www.live2d.com/en/download/cubism-sdk/download-native/).

#![warn(missing_docs)]

pub mod drawable;
pub mod log;
pub mod model;
pub mod parameter;
pub mod part;

mod error;
mod flags;
mod moc;
mod version;

pub use error::*;
pub use flags::*;
pub use moc::*;
pub use model::Model;
pub use version::*;

/// Necessary alignment for mocs (in bytes).
pub(crate) const ALIGN_OF_MOC: usize = cubism_core_sys::csmAlignofMoc as _;
/// Necessary alignment for models (in bytes).
pub(crate) const ALIGN_OF_MODEL: usize = cubism_core_sys::csmAlignofModel as _;

/// A trait for getting data from [`Model`].
pub trait ModelData {
    /// Data type.
    type Data;

    /// The count of [`Data`](Self::Data)
    fn count(&self) -> usize;

    /// Returns the index of [`Data`](Self::Data) according to its ID,
    /// or returns [`None`] if ID doesn't exist.
    fn index<T: AsRef<str>>(&self, id: T) -> Option<usize>;

    /// Returns [`Data`](Self::Data) accouding to its index.
    ///
    /// # Safety
    ///
    /// The caller should make sure the index isn't out of bound.
    unsafe fn get_index_unchecked(&self, index: usize) -> Self::Data;

    /// Returns [`Data`](Self::Data) accouding to its ID.
    ///
    /// # Panics
    ///
    /// Panics if the ID doesn't exist.
    #[inline]
    fn get<T: AsRef<str>>(&self, id: T) -> Self::Data {
        // SAFETY: the index from `index()` is never out of bound.
        unsafe {
            self.get_index_unchecked(
                self.index(id.as_ref())
                    .unwrap_or_else(|| panic!("ID {} doesn't exist", id.as_ref())),
            )
        }
    }

    /// Returns [`Data`](Self::Data) accouding to its index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bound.
    #[inline]
    fn get_index(&self, index: usize) -> Self::Data {
        assert!(index < self.count());
        // SAFETY: the index has been checked.
        unsafe { self.get_index_unchecked(index) }
    }
}

macro_rules! impl_iter {
    ($iter:ty, $item:ty, $collect:ty) => {
        impl<'a> std::iter::Iterator for $iter {
            type Item = $item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.start < self.end {
                    // SAFETY: the index has been checked.
                    unsafe {
                        let data = self.get_index_unchecked(self.start);
                        self.start += 1;
                        Some(data)
                    }
                } else {
                    None
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let remain = self.end - self.start;
                (remain, Some(remain))
            }
        }

        impl<'a> std::iter::DoubleEndedIterator for $iter {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.start < self.end {
                    self.end -= 1;
                    // SAFETY: it's never out of bound.
                    unsafe { Some(self.get_index_unchecked(self.end)) }
                } else {
                    None
                }
            }
        }

        impl<'a> std::iter::ExactSizeIterator for $iter {}
        impl<'a> std::iter::FusedIterator for $iter {}

        impl<'a> $iter {
            /// Gets all data.
            #[inline]
            pub fn get_all(self) -> $collect {
                self.collect()
            }
        }
    };
}

pub(crate) use impl_iter;

#[cfg(test)]
pub(crate) fn read_haru_moc() -> Result<moc::Moc> {
    use std::env;
    use std::path::PathBuf;

    let samples_dir = env::var("LIVE2D_CUBISM").expect(
        "The environment variable `LIVE2D_CUBISM` is not set properly. \
        `LIVE2D_CUBISM` should be set to the Live2D Cubism directory.",
    );
    let mut haru_moc = PathBuf::from(samples_dir);
    haru_moc.push("Samples");
    haru_moc.push("Resources");
    haru_moc.push("Haru");
    haru_moc.push("Haru.moc3");

    moc::Moc::from_file(haru_moc)
}
