use crate::{Error, MocVersion, Result, ALIGN_OF_MOC};
use aligned_utils::bytes::AlignedBytes;
use std::{fs::File, io::Read, os::raw::c_uint, path::Path, sync::Arc};

/// Cubism moc.
#[derive(Clone, Debug)]
pub struct Moc {
    moc: Arc<AlignedBytes>,
}

#[inline]
fn get_moc_version(data: &AlignedBytes) -> cubism_core_sys::csmMocVersion {
    unsafe { cubism_core_sys::csmGetMocVersion(data.as_ptr().cast(), data.len() as _) }
}

impl Moc {
    /// Creates [`Moc`].
    pub fn new<T: AsRef<[u8]>>(moc3_data: T) -> Result<Self> {
        if moc3_data.as_ref().len() > c_uint::MAX as _ {
            return Err(Error::MocDataTooLarge);
        }
        let mut data = AlignedBytes::new_from_slice(moc3_data.as_ref(), ALIGN_OF_MOC);
        debug_assert_eq!(data.len(), moc3_data.as_ref().len());
        let version = get_moc_version(&data);

        unsafe {
            if MocVersion::from(version) > MocVersion::latest_version() {
                Err(Error::InvalidMocVersion(version))
            } else if cubism_core_sys::csmReviveMocInPlace(
                data.as_mut_ptr().cast(),
                data.len() as _,
            )
            .is_null()
            {
                Err(Error::InvalidMocData)
            } else {
                Ok(Self {
                    moc: Arc::new(data),
                })
            }
        }
    }

    /// Creates [`Moc`] from `moc3` file.
    #[inline]
    pub fn from_file<T: AsRef<Path>>(moc3_file: T) -> Result<Self> {
        let mut file = File::open(moc3_file)?;
        let mut data = Vec::new();
        let _ = file.read_to_end(&mut data)?;

        Self::new(data)
    }

    /// Gets [`Moc`] format version.
    #[inline]
    pub fn version(&self) -> MocVersion {
        get_moc_version(&self.moc).into()
    }

    /// Gets the size of moc.
    #[inline]
    pub fn moc_size(&self) -> usize {
        self.moc.len()
    }

    /// Converts [`Moc`] to a [`cubism_core_sys::csmMoc`] pointer.
    /// The caller should make sure the pointer won't live longer than [`Moc`].
    #[inline]
    pub fn as_moc_ptr(&self) -> *const cubism_core_sys::csmMoc {
        self.moc.as_ptr().cast()
    }
}

impl std::convert::TryFrom<&[u8]> for Moc {
    type Error = Error;

    #[inline]
    fn try_from(data: &[u8]) -> Result<Self> {
        Self::new(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        log::{set_logger, DefaultLogger},
        read_haru_moc,
    };

    #[test]
    fn test_moc() -> Result<()> {
        set_logger(DefaultLogger);
        let moc = read_haru_moc()?;
        assert!(moc.version().is_version30());

        Ok(())
    }
}
