/// Cubism Core version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CubismVersion {
    pub version: u32,
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl CubismVersion {
    /// Queries Cubism Core version.
    #[inline]
    pub fn version() -> Self {
        let version = unsafe { cubism_core_sys::csmGetVersion() };

        Self {
            version,
            major: ((version & 0xFF00_0000) >> 24) as _,
            minor: ((version & 0x00FF_0000) >> 16) as _,
            patch: (version & 0x0000_FFFF) as _,
        }
    }
}

impl std::fmt::Display for CubismVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{} ({})",
            self.major, self.minor, self.patch, self.version
        )
    }
}

/// `moc3` file format version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MocVersion {
    /// `moc3` file version 3.0.00 - 3.2.07
    Version30,
    /// `moc3` file version 3.3.00 - 3.3.03
    Version33,
    /// `moc3` file version 4.0.00
    Version40,
    /// unknown `moc3` file version
    VersionUnknown,
}

impl MocVersion {
    /// Creates [`MocVersion`].
    #[inline]
    pub fn new(version: cubism_core_sys::csmMocVersion) -> Self {
        match version {
            1 => MocVersion::Version30,
            2 => MocVersion::Version33,
            3 => MocVersion::Version40,
            _ => MocVersion::VersionUnknown,
        }
    }

    /// Gets latest version which `moc3` file is supported.
    #[inline]
    pub fn latest_version() -> Self {
        unsafe { cubism_core_sys::csmGetLatestMocVersion().into() }
    }

    /// Returns `true` if the [`MocVersion`] is [`Version30`](MocVersion::Version30).
    #[inline]
    pub fn is_version30(self) -> bool {
        self == Self::Version30
    }

    /// Returns `true` if the [`MocVersion`] is [`Version33`](MocVersion::Version33).
    #[inline]
    pub fn is_version33(self) -> bool {
        self == Self::Version33
    }

    /// Returns `true` if the [`MocVersion`] is [`Version40`](MocVersion::Version40).
    #[inline]
    pub fn is_version40(self) -> bool {
        self == Self::Version40
    }

    /// Returns `true` if the [`MocVersion`] is [`VersionUnknown`](MocVersion::VersionUnknown).
    #[inline]
    pub fn is_version_unknown(self) -> bool {
        self == Self::VersionUnknown
    }
}

impl From<cubism_core_sys::csmMocVersion> for MocVersion {
    #[inline]
    fn from(version: cubism_core_sys::csmMocVersion) -> Self {
        Self::new(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubism_version() {
        let version = CubismVersion::version();
        assert_eq!(
            version.version,
            ((version.major as u32) << 24) + ((version.minor as u32) << 16) + version.patch as u32
        );
    }

    #[test]
    fn test_moc_version() {
        let latest_version = MocVersion::latest_version();
        assert!(latest_version.is_version40());
    }
}
