//! The `galois` module provides functionality for performing operations in Galois fields.
//!
//! Galois fields are finite fields commonly used in error correction codes, cryptography,
//! and other applications requiring mathematical operations over a finite set of elements.
//!
//! This module is designed to interface with low-level Galois field operations, provided by
//! library `gf-complete`.

use crate::{CodeWord, Error, MACHINE_LONG_SIZE};

/// The `GaloisField` struct represents a Galois field GF(2^w) with a specified word size `w`.
///
/// It provides methods for performing various operations in the Galois field, such as
/// addition, multiplication, division, and inversion.
///
/// # Note
/// - The word size `w` must be in the range 1..=32.
/// - All the slices passed to the methods must be multiples of machine `long` size.
pub struct GaloisField {
    w: CodeWord,
}

impl GaloisField {
    /// Creates a new GaloisField with the specified word size.
    /// The word size must be in range 1..=32.
    pub fn try_from_code_word(w: CodeWord) -> Option<Self> {
        let w_u8 = w.to_u8();
        if w_u8 == 0 || w_u8 > 32 {
            return None;
        }
        unsafe { jerasure_sys::jerasure::galois_init_default_field(w_u8 as i32) };
        Some(GaloisField { w })
    }

    /// Returns the word size of the GaloisField.
    pub fn get_w(&self) -> CodeWord {
        CodeWord::from_u8(self.w.to_u8())
    }

    /// Returns the inverse of `a` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// # use jerasure_rs::CodeWord;
    /// let gf = GaloisField::try_from_code_word(CodeWord::W8).unwrap();
    /// assert_eq!(gf.inverse(142), 2);
    /// ```
    /// # Note: This is not the same as `1 / a` in normal arithmetic.
    pub fn inverse(&self, a: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_inverse(a, self.w.as_cint()) }
    }

    /// Returns the result of `a + b` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// # use jerasure_rs::CodeWord;
    /// let gf = GaloisField::try_from_code_word(CodeWord::W8).unwrap();
    /// assert_eq!(gf.add(24, 54), 46);
    /// ```
    /// # Note: This is not the same as `a + b` in normal arithmetic.
    pub fn add(&self, a: i32, b: i32) -> i32 {
        a ^ b
    }

    /// Returns the result of `a * b` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// # use jerasure_rs::CodeWord;
    /// let gf = GaloisField::try_from_code_word(CodeWord::W8).unwrap();
    /// assert_eq!(gf.multiply(24, 84), 179);
    /// ```
    /// # Note: This is not the same as `a * b` in normal arithmetic.
    pub fn multiply(&self, a: i32, b: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_single_multiply(a, b, self.w.as_cint()) }
    }

    /// Returns the result of `a / b` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// # use jerasure_rs::CodeWord;
    /// let gf = GaloisField::try_from_code_word(CodeWord::W8).unwrap();
    /// assert_eq!(gf.divide(23, 74), 91);
    /// ```
    /// # Note: This is not the same as `a / b` in normal arithmetic.
    pub fn divide(&self, a: i32, b: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_single_divide(a, b, self.w.as_cint()) }
    }

    /// Calculates the result of `a + b` in the GF(2^w) and stores it in `out`.
    ///
    /// That is, `out[i] = a[i] + b[i]`.
    ///
    /// Use [region_acc](Self::region_acc) if you want to accumulate the result in `a`.
    pub fn region_add(
        &self,
        a: impl AsRef<[u8]>,
        b: impl AsRef<[u8]>,
        mut out: impl AsMut<[u8]>,
    ) -> Result<(), Error> {
        let a = a.as_ref();
        let b = b.as_ref();
        let out = out.as_mut();
        let n = a.len();
        if n != b.len() {
            return Err(Error::invalid_arguments(format!(
                "Input slices must be the same length: a.len({}) != b.len({})",
                a.len(),
                b.len()
            )));
        }
        if n != out.len() {
            return Err(Error::invalid_arguments(format!(
                "Output slice must be the same length as input slices: out.len({}) != a.len({})",
                out.len(),
                a.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::NotAligned(n));
        }
        out.copy_from_slice(b);
        unsafe {
            jerasure_sys::jerasure::galois_region_xor(
                a.as_ptr() as *mut ::std::os::raw::c_char,
                out.as_mut_ptr() as *mut ::std::os::raw::c_char,
                n.try_into().unwrap(),
            );
        }
        Ok(())
    }

    /// Calculates the result of `buf + acc` in the GF(2^w) and stores it in `buf`.
    ///
    /// That is, `buf[i] = buf[i] + acc[i]`.
    pub fn region_acc(
        &self,
        mut buf: impl AsMut<[u8]>,
        acc: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        let src: &[u8] = acc.as_ref();
        let dest = buf.as_mut();
        let n = src.len();
        if n != dest.len() {
            return Err(Error::invalid_arguments(format!(
                "Output slice must be the same length as input slices: out.len({}) != a.len({})",
                dest.len(),
                src.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::NotAligned(n));
        }
        unsafe {
            jerasure_sys::jerasure::galois_region_xor(
                src.as_ptr() as *mut ::std::os::raw::c_char,
                dest.as_mut_ptr() as *mut ::std::os::raw::c_char,
                n.try_into().unwrap(),
            );
        }
        Ok(())
    }

    /// Multiplies the `src` slice by `multiply_by` and adds `add` to each element, storing the result in `dest`.
    ///
    /// That is, `dest[i] = src[i] * multiply_by + add`.
    ///
    /// # Panics
    /// It only works for word sizes of 8, 16, or 32 bits, otherwise it will panic.
    pub fn region_multiply(
        &self,
        src: impl AsRef<[u8]>,
        multiply_by: i32,
        add: i32,
        mut dest: impl AsMut<[u8]>,
    ) -> Result<(), Error> {
        let src = src.as_ref();
        let dest = dest.as_mut();
        let n = src.len();
        if n != dest.len() {
            return Err(Error::invalid_arguments(format!(
                "Input slices must be the same length: src.len({}) != dest.len({})",
                src.len(),
                dest.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::NotAligned(n));
        }
        let mul_fn = match self.w {
            CodeWord::W8 => jerasure_sys::jerasure::galois_w08_region_multiply,
            CodeWord::W16 => jerasure_sys::jerasure::galois_w16_region_multiply,
            CodeWord::W32 => jerasure_sys::jerasure::galois_w32_region_multiply,
            CodeWord::Other(_) => {
                return Err(Error::not_supported(
                    "region multiply only supports w in {8, 16, 32}",
                ));
            }
        };
        let src_ptr = src.as_ptr() as *mut ::std::os::raw::c_char;
        let dest_ptr = dest.as_mut_ptr() as *mut ::std::os::raw::c_char;
        unsafe {
            mul_fn(src_ptr, multiply_by, n.try_into().unwrap(), dest_ptr, add);
        }
        Ok(())
    }
}
