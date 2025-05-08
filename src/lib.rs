/// The `galois` module provides functionality for performing operations in Galois fields.
///
/// Galois fields are finite fields commonly used in error correction codes, cryptography,
/// and other applications requiring mathematical operations over a finite set of elements.
///
/// This module is designed to interface with low-level Galois field operations, provided by
/// library `gf-complete`.
pub mod galois;

const MACHINE_LONG_SIZE: usize = size_of::<std::os::raw::c_long>();

/// The `Error` enum defines the possible errors that this crate can occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// InvalidRange: The range of the input is invalid.
    #[error("Invalid Range: {0}")]
    InvalidRange(String),
    /// InvalidWordSize: The word size is not a multiple of the machine long size.
    #[error("Invalid Word Size: {0} is not multiple of {MACHINE_LONG_SIZE}")]
    InvalidWordSize(usize),
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
