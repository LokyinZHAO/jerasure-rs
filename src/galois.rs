use crate::{Error, MACHINE_LONG_SIZE};

/// The `GaloisField` struct represents a Galois field GF(2^w) with a specified word size `w`.
///
/// It provides methods for performing various operations in the Galois field, such as
/// addition, multiplication, division, and inversion.
///
/// # Note
/// - The word size `w` must be in the range 1..=32.
/// - All the slices passed to the methods must be multiples of machine `long` size.
pub struct GaloisField {
    w: u8,
}

impl GaloisField {
    /// Creates a new GaloisField with the specified word size.
    /// The word size must be in range 1..=32.
    pub fn try_from_word_size(w: u8) -> Option<Self> {
        if w == 0 || w > 32 {
            return None;
        }
        unsafe { jerasure_sys::jerasure::galois_init_default_field(w as i32) };
        Some(GaloisField { w })
    }

    /// Returns the word size of the GaloisField.
    pub fn get_w(&self) -> u8 {
        self.w
    }

    /// Returns the inverse of `a` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// let gf = GaloisField::try_from_word_size(8).unwrap();
    /// assert_eq!(gf.inverse(142), 2);
    /// ```
    /// # Note: This is not the same as `1 / a` in normal arithmetic.
    pub fn inverse(&self, a: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_inverse(a, self.w as i32) }
    }

    /// Returns the result of `a + b` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// let gf = GaloisField::try_from_word_size(8).unwrap();
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
    /// let gf = GaloisField::try_from_word_size(8).unwrap();
    /// assert_eq!(gf.multiply(24, 84), 179);
    /// ```
    /// # Note: This is not the same as `a * b` in normal arithmetic.
    pub fn multiply(&self, a: i32, b: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_single_multiply(a, b, self.w as i32) }
    }

    /// Returns the result of `a / b` in the GF(2^w).
    /// # Example
    /// ```
    /// # use jerasure_rs::galois::GaloisField;
    /// let gf = GaloisField::try_from_word_size(8).unwrap();
    /// assert_eq!(gf.divide(23, 74), 91);
    /// ```
    /// # Note: This is not the same as `a / b` in normal arithmetic.
    pub fn divide(&self, a: i32, b: i32) -> i32 {
        unsafe { jerasure_sys::jerasure::galois_single_divide(a, b, self.w as i32) }
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
            return Err(Error::InvalidRange(format!(
                "Input slices must be the same length: a.len({}) != b.len({})",
                a.len(),
                b.len()
            )));
        }
        if n != out.len() {
            return Err(Error::InvalidRange(format!(
                "Output slice must be the same length as input slices: out.len({}) != a.len({})",
                out.len(),
                a.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::InvalidWordSize(n));
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
            return Err(Error::InvalidRange(format!(
                "Output slice must be the same length as input slices: out.len({}) != a.len({})",
                dest.len(),
                src.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::InvalidWordSize(n));
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
    /// That is,  `dest[i] = src[i] * multiply_by + add`.
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
            return Err(Error::InvalidRange(format!(
                "Input slices must be the same length: src.len({}) != dest.len({})",
                src.len(),
                dest.len()
            )));
        }
        if n % MACHINE_LONG_SIZE != 0 {
            return Err(Error::InvalidWordSize(n));
        }
        let mul_fn = match self.w {
            8 => jerasure_sys::jerasure::galois_w08_region_multiply,
            16 => jerasure_sys::jerasure::galois_w16_region_multiply,
            32 => jerasure_sys::jerasure::galois_w32_region_multiply,
            _ => unimplemented!("only support w=8, 16, 32"),
        };
        let src_ptr = src.as_ptr() as *mut ::std::os::raw::c_char;
        let dest_ptr = dest.as_mut_ptr() as *mut ::std::os::raw::c_char;
        unsafe {
            mul_fn(src_ptr, multiply_by, n.try_into().unwrap(), dest_ptr, add);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_ctor() {
        let gf = super::GaloisField::try_from_word_size(8);
        assert!(gf.is_some());
        let gf = super::GaloisField::try_from_word_size(16);
        assert!(gf.is_some());
        let gf = super::GaloisField::try_from_word_size(32);
        assert!(gf.is_some());
        let gf = super::GaloisField::try_from_word_size(0);
        assert!(gf.is_none());
        let gf = super::GaloisField::try_from_word_size(33);
        assert!(gf.is_none());
    }

    #[test]
    fn test_w8_region_mult() {
        let gf = super::GaloisField::try_from_word_size(8).unwrap();
        let src = [
            0xc4, 0xfa, 0x87, 0xee, 0x9a, 0x57, 0xcd, 0x56, 0xe2, 0xc2, 0xea, 0x11, 0xcc, 0x59,
            0x84, 0x26,
        ];
        let expect_out = [
            0x90, 0x27, 0xba, 0xfe, 0xae, 0xf3, 0x5d, 0x1d, 0x42, 0xce, 0x61, 0xa8, 0xb3, 0x8e,
            0x95, 0xd2,
        ];
        let mut out = [0_u8; 16];
        gf.region_multiply(src.as_slice(), 238, 0, &mut out)
            .unwrap();
        assert_eq!(expect_out, out);

        let src = [
            0xe4, 0x6e, 0xc4, 0x84, 0xc8, 0xc1, 0x13, 0x04, 0x68, 0x76, 0x01, 0x09, 0x12, 0x7d,
            0x82, 0xaa,
        ];
        let expect_out = [
            0x3a, 0x35, 0x25, 0x1b, 0x8c, 0x92, 0xec, 0x67, 0xef, 0x7a, 0xd0, 0x1e, 0x3c, 0xd9,
            0xc1, 0x10,
        ];
        let mut out = [0_u8; 16];
        gf.region_multiply(src.as_slice(), 208, 80, &mut out)
            .unwrap();
        assert_eq!(expect_out, out);
    }

    #[test]
    fn test_w8_region_xor() {
        let gf = super::GaloisField::try_from_word_size(8).unwrap();
        let src_a = [0xc4, 0xfa, 0x87, 0xee, 0x9a, 0x57, 0xcd, 0x56];
        let src_b = [0x9a, 0x57, 0xcd, 0x56, 0xc4, 0xfa, 0x87, 0xee];
        let expect_out = [0x5e, 0xad, 0x4a, 0xb8, 0x5e, 0xad, 0x4a, 0xb8];
        let mut out = [0_u8; 8];
        gf.region_add(src_a.as_slice(), src_b.as_slice(), &mut out)
            .unwrap();
        assert_eq!(expect_out, out);
        let mut buf = src_a.clone();
        let acc = src_b.clone();
        gf.region_acc(&mut buf, &acc).unwrap();
        assert_eq!(buf, expect_out);
    }
}
