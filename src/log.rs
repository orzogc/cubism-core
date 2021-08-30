use std::ffi::CStr;

type LogFunction = unsafe extern "C" fn(message: *const i8);

pub trait Logger {
    fn log(message: &str);

    unsafe extern "C" fn log_callback(message: *const i8) {
        Self::log(CStr::from_ptr(message).to_string_lossy().as_ref())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DefaultLogger;

impl Logger for DefaultLogger {
    #[inline]
    fn log(message: &str) {
        println!("{}", message);
    }
}

pub fn set_logger<T: Logger>(_: T) {
    unsafe {
        cubism_core_sys::csmSetLogFunction(Some(T::log_callback));
    }
}

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
