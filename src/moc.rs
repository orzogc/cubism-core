use crate::error::{Error, Result};
use crate::version::MocVersion;
use crate::ALIGN_OF_MOC;
use aligned_utils::bytes::AlignedBytes;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

/// Cubism moc.
#[derive(Clone, Debug)]
pub struct Moc {
    moc: Arc<AlignedBytes>,
    size: usize,
}

#[inline]
fn get_moc_version(data: &AlignedBytes, size: usize) -> cubism_core_sys::csmMocVersion {
    unsafe { cubism_core_sys::csmGetMocVersion(data.as_ptr() as _, size as _) }
}

impl Moc {
    pub fn new<T: AsRef<[u8]>>(moc3_data: T) -> Result<Self> {
        let mut data = AlignedBytes::new_from_slice(moc3_data.as_ref(), ALIGN_OF_MOC);
        let size = moc3_data.as_ref().len();

        let version = get_moc_version(&data, size);

        if MocVersion::version(version) > MocVersion::get_latest_version() {
            Err(Error::InvalidMocVersion(version))
        } else if unsafe { cubism_core_sys::csmReviveMocInPlace(data.as_mut_ptr() as _, size as _) }
            .is_null()
        {
            Err(Error::InvalidMocData)
        } else {
            Ok(Self {
                moc: Arc::new(data),
                size,
            })
        }
    }

    pub fn from_file<T: AsRef<Path>>(moc3_file: T) -> Result<Self> {
        let mut file = File::open(moc3_file)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Self::new(data)
    }

    #[inline]
    pub fn moc(&self) -> Arc<AlignedBytes> {
        self.moc.clone()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets Moc file format version.
    #[inline]
    pub fn version(&self) -> MocVersion {
        MocVersion::version(get_moc_version(&self.moc, self.size))
    }

    #[inline]
    pub(crate) fn as_moc_ptr(&self) -> *const cubism_core_sys::csmMoc {
        self.moc.as_ptr() as _
    }
}

/*
impl core::ops::Deref for Moc {
    type Target = AlignedBytes;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<AlignedBytes> for Moc {
    #[inline]
    fn as_ref(&self) -> &AlignedBytes {
        &self.data
    }
}
*/

impl From<AlignedBytes> for Moc {
    #[inline]
    fn from(bytes: AlignedBytes) -> Self {
        let size = bytes.len();

        Self {
            moc: Arc::new(bytes),
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_haru_moc;

    #[test]
    fn test_moc() -> Result<()> {
        let moc = read_haru_moc()?;
        assert!(moc.version().is_version30());

        Ok(())
    }
}
