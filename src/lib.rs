pub mod erasure;
pub mod galois;

const MACHINE_LONG_SIZE: usize = size_of::<std::os::raw::c_long>();

#[derive(Debug, Clone, Copy, Default)]
pub enum CodeWord {
    #[default]
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
    #[error("Too Many Erased Blocks: {0} erased, up to {1} allowed")]
    TooManyErasure(i32, i32),
    /// InvalidArguments: The the input is invalid.
    #[error("Invalid Arguments: {0}")]
    InvalidArguments(String),
    /// NotAligned: The input is not a multiple of the machine long size.
    #[error("Not Aligned: {0} is not multiple of {MACHINE_LONG_SIZE}")]
    NotAligned(usize),
    /// NotSupported: The input is not supported.
    #[error("Not Supported: {0}")]
    NotSupported(String),
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    fn too_many_erasure(erasures: i32, max_erasures: i32) -> Self {
        Self::TooManyErasure(erasures, max_erasures)
    }

    fn invalid_arguments(msg: impl Into<String>) -> Self {
        Self::InvalidArguments(msg.into())
    }

    fn not_supported(msg: impl Into<String>) -> Self {
        Self::NotSupported(msg.into())
    }

    fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
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
