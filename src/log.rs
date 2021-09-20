//! Logger for the Cubism Core lib.

use std::{borrow::Cow, ffi::CStr, os::raw::c_char};

/// Log function type.
pub type LogFunction = unsafe extern "C" fn(message: *const c_char);

/// Logger trait.
/// Implementing this trait for setting the logger in the Cubism Core lib.
pub trait Logger {
    /// Log the message.
    fn log<'a>(message: impl Into<Cow<'a, str>>);

    /// Log function for the Cubism Core lib to callback.
    /// For most cases, there's no need to implement it.
    ///
    /// # Safety
    ///
    /// `message` is a pointer to a C string.
    #[inline]
    unsafe extern "C" fn log_callback(message: *const c_char) {
        Self::log(CStr::from_ptr(message).to_string_lossy())
    }
}

/// Default logger. Just print message to the console.
#[derive(Clone, Copy, Debug)]
pub struct DefaultLogger;

impl Logger for DefaultLogger {
    #[inline]
    fn log<'a>(message: impl Into<Cow<'a, str>>) {
        println!("cubism: {}", message.into());
    }
}

/// Set the logger in the Cubism Core lib.
#[inline]
pub fn set_logger<T: Logger>(_: T) {
    unsafe {
        cubism_core_sys::csmSetLogFunction(Some(T::log_callback));
    }
}

/// Gets the logger function in the Cubism Core lib.
#[inline]
pub fn get_logger() -> Option<LogFunction> {
    unsafe { cubism_core_sys::csmGetLogFunction() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        assert!(get_logger().is_none());
        set_logger(DefaultLogger);
        assert!(get_logger().is_some());
    }
}
