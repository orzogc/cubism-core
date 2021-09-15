//#![warn(missing_docs)]

pub mod log;
pub mod model;

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
