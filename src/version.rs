/// Cubism version identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CubismVersion {
    pub version_number: usize,
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl CubismVersion {
    /// Queries Cubism Core version.
    #[inline]
    pub fn version() -> Self {
        let version_number = unsafe { cubism_core_sys::csmGetVersion() };

        Self {
            version_number: version_number as _,
            major: ((version_number & 0xFF00_0000) >> 24) as _,
            minor: ((version_number & 0x00FF_0000) >> 16) as _,
            patch: (version_number & 0x0000_FFFF) as _,
        }
    }
}

impl core::fmt::Display for CubismVersion {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{}.{} ({})",
            self.major, self.minor, self.patch, self.version_number
        )
    }
}

/// moc3 version identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MocVersion {
    /// moc3 file version 3.0.00 - 3.2.07
    Version30,
    /// moc3 file version 3.3.00 - 3.3.03
    Version33,
    /// moc3 file version 4.0.00
    Version40,
    /// unknown moc3 file version
    VersionUnknown,
}

impl MocVersion {
    #[inline]
    pub(crate) fn version(v: cubism_core_sys::csmMocVersion) -> Self {
        match v {
            1 => MocVersion::Version30,
            2 => MocVersion::Version33,
            3 => MocVersion::Version40,
            _ => MocVersion::VersionUnknown,
        }
    }

    /// Gets Moc file supported latest version.
    #[inline]
    pub fn get_latest_version() -> Self {
        Self::version(unsafe { cubism_core_sys::csmGetLatestMocVersion() })
    }

    /// Returns `true` if the `MocVersion` is [`Version30`](MocVersion::Version30).
    #[inline]
    pub fn is_version30(&self) -> bool {
        matches!(self, Self::Version30)
    }

    /// Returns `true` if the `MocVersion` is [`Version33`](MocVersion::Version33).
    #[inline]
    pub fn is_version33(&self) -> bool {
        matches!(self, Self::Version33)
    }

    /// Returns `true` if the `MocVersion` is [`Version40`](MocVersion::Version40).
    #[inline]
    pub fn is_version40(&self) -> bool {
        matches!(self, Self::Version40)
    }

    /// Returns `true` if the `MocVersion` is [`VersionUnknown`](MocVersion::VersionUnknown).
    #[inline]
    pub fn is_version_unknown(&self) -> bool {
        matches!(self, Self::VersionUnknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubism_version() {
        let version = CubismVersion::version();
        assert_eq!(
            version.version_number as usize,
            (version.major << 24) + (version.minor << 16) + version.patch
        );
    }

    #[test]
    fn test_moc_version() {
        let latest_version = MocVersion::get_latest_version();
        assert!(latest_version.is_version40());
    }
}
