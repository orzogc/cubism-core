/// `Result` for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// `Error` for this crate.
#[derive(Debug)]
pub enum Error {
    /// Invalid `moc3` file format version.
    InvalidMocVersion(cubism_core_sys::csmMocVersion),
    /// The size of `moc3` data is larger than [`u32::MAX`](https://doc.rust-lang.org/std/primitive.u32.html#associatedconstant.MAX).
    MocDataTooLarge,
    /// Invalid `moc3` data.
    InvalidMocData,
    /// Failed to initialize model.
    InitializeModelError,
    /// Invalid count.
    InvalidCount(&'static str),
    /// Failed to get data.
    GetDataError(&'static str),
    /// Invalid flags.
    InvalidFlags(&'static str, u8),
    /// Two slices have different lengths.
    SliceLengthNotEqual(usize, usize),
    /// Failed to read/write file.
    FileIoError(std::io::Error),
}

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMocVersion(v) => write!(f, "unsupported moc version: {}", v),
            Error::MocDataTooLarge => write!(f, "the size of moc3 data is too large"),
            Error::InvalidMocData => write!(f, "invalid moc3 data"),
            Error::InitializeModelError => write!(f, "failed to initialize model"),
            Error::InvalidCount(s) => write!(f, "invalid count of {}", s),
            Error::GetDataError(s) => write!(f, "failed to get {}", s),
            Error::InvalidFlags(s, u) => write!(f, "invalid {} flags: {}", s, u),
            Error::SliceLengthNotEqual(len1, len2) => {
                write!(f, "two slices have different lengths: {}, {}", len1, len2)
            }
            Error::FileIoError(e) => write!(f, "{}", *e),
        }
    }
}

impl std::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidMocVersion(_) => None,
            Error::MocDataTooLarge => None,
            Error::InvalidMocData => None,
            Error::InitializeModelError => None,
            Error::InvalidCount(_) => None,
            Error::GetDataError(_) => None,
            Error::InvalidFlags(_, _) => None,
            Error::SliceLengthNotEqual(_, _) => None,
            Error::FileIoError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        Error::FileIoError(err)
    }
}
