pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidMocVersion(cubism_core_sys::csmMocVersion),
    InvalidMocData,
    InitializeModelError,
    InvalidDataCount(&'static str),
    GetDataError(&'static str),
    InvalidFlags(&'static str, u8),
    FileIoError(std::io::Error),
}

impl core::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMocVersion(v) => write!(f, "unsupported moc version: {}", *v),
            Error::InvalidMocData => write!(f, "invalid moc data"),
            Error::InitializeModelError => write!(f, "failed to initialize model"),
            Error::InvalidDataCount(s) => write!(f, "invalid {} count", *s),
            Error::GetDataError(s) => write!(f, "failed to get {}", *s),
            Error::InvalidFlags(s, u) => write!(f, "invalid {} flags: {}", *s, *u),
            Error::FileIoError(e) => write!(f, "{}", *e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        Error::FileIoError(err)
    }
}
