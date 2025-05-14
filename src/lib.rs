pub mod erasure;
pub mod galois;

const MACHINE_LONG_SIZE: usize = size_of::<std::os::raw::c_long>();

#[derive(Debug, Clone, Copy, Default)]
/// The `CodeWord` is used to represent the size of the code word in bits.
///
/// The `CodeWord` enum defines the possible code words that can be used in galois fied.
/// And the default value is `W8`, which is the most common code word size.
pub enum CodeWord {
    #[default]
    /// A code word of 1 Byte.
    W8,
    /// A code word of 2 Bytes.
    W16,
    /// A code word of 4 Bytes.
    W32,
    /// A code word of other size in bits.
    Other(u8),
}

impl CodeWord {
    /// Makes a new `CodeWord` from the given size in bits.
    pub fn from_u8(w: u8) -> Self {
        match w {
            8 => Self::W8,
            16 => Self::W16,
            32 => Self::W32,
            _ => Self::Other(w),
        }
    }

    /// Returns the size of the code word in bits.
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
    /// TooManyErasure: The number of erasures is larger than the maximum allowed,
    /// and the lost data cannot be recovered.
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
    /// Other: Other errors that are not covered by the above.
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
