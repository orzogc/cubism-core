pub mod error;
pub mod flags;
pub mod log;
pub mod moc;
pub mod model;
pub mod version;

/// Necessary alignment for mocs (in bytes).
pub(crate) const ALIGN_OF_MOC: usize = cubism_core_sys::csmAlignofMoc as _;
/// Necessary alignment for models (in bytes).
pub(crate) const ALIGN_OF_MODEL: usize = cubism_core_sys::csmAlignofModel as _;

#[cfg(test)]
pub fn read_haru_moc() -> error::Result<moc::Moc> {
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
