pub mod galois;

const MACHINE_LONG_SIZE: usize = size_of::<std::os::raw::c_long>();

pub enum CodeWord {
    W8,
    W16,
    W32,
    Other(u8),
}

impl CodeWord {
    pub fn from_u8(w: u8) -> Self {
        match w {
            8 => Self::W8,
            16 => Self::W16,
            32 => Self::W32,
            _ => Self::Other(w),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::W8 => 8,
            Self::W16 => 16,
            Self::W32 => 32,
            Self::Other(w) => *w,
        }
    }

    fn as_cint(&self) -> ::std::ffi::c_int {
        match self {
            Self::W8 => 8,
            Self::W16 => 16,
            Self::W32 => 32,
            Self::Other(w) => *w as ::std::ffi::c_int,
        }
    }
}

/// The `Error` enum defines the possible errors that this crate can occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// InvalidArguments: The the input is invalid.
    #[error("Invalid Arguments: {0}")]
    InvalidArguments(String),
    /// NotAligned: The input is not a multiple of the machine long size.
    #[error("Not Aligned: {0} is not multiple of {MACHINE_LONG_SIZE}")]
    NotAligned(usize),
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[derive(Debug)]
struct MallocBox<T> {
    ptr: *mut T,
}

impl<T> MallocBox<T> {
    /// Make a malloc box from a pointer from `malloc`.
    ///
    /// # Safety
    /// This function is unsafe because improper use may lead to memory problems. For example,
    /// a double-free may occur if the function is called twice on the same raw pointer.
    unsafe fn try_from_raw(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        Some(Self { ptr })
    }

    fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl<T> Drop for MallocBox<T> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut libc::c_void);
        }
    }
}

#[cfg(test)]
mod tests {
    use jerasure_sys;

    #[test]
    fn link_works() {
        unsafe {
            jerasure_sys::jerasure::galois_init_default_field(8);
        }
    }
}
