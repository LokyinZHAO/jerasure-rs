//! The `erasure` module provides an interface for encoding and decoding data using erasure codes.
//!
//! This module is designed to bind with the Jerasure library, which provides efficient
//! implementations of various erasure coding techniques.
//!
//! For more information, see the [jerasure documentation](https://github.com/tsuraan/Jerasure/blob/414c96ef2b9934953b6facb31d803d79b1dd1405/Manual.pdf)

use ::std::os::raw::c_int;
use std::num::NonZeroI32;

use crate::{CodeWord, Error};

use iter_tools::Itertools;

#[derive(Debug, Clone, Copy, Default)]
/// The `Technique` is used to represent the technique used to encode and decode the data.
///
/// For more information, see the [jerasure documentation](https://github.com/tsuraan/Jerasure/blob/414c96ef2b9934953b6facb31d803d79b1dd1405/Manual.pdf)
///
/// The Default value is `Matrix`, which is the most basic technique used in galois field.
/// But the `BitMatrix` and `Schedule` techniques are more efficient in the most cases.
/// And the `ScheduleCache` technique is a `Schedule` technique with a cache version,
/// which is more efficient than the `Schedule` technique but requires more memory.
pub enum Technique {
    #[default]
    /// The matrix coding technique.
    ///
    /// # Requires
    /// - w must be in {8,16,32}
    Matrix,
    /// The bit-matrix coding technique.
    ///
    /// # Requires
    /// - packet_size must be set
    /// - w not greater than 32
    /// - not supported for ReedSolVand
    BitMatrix,
    /// The schedule coding technique.
    ///
    /// # Requires
    /// - packet_size must be set
    /// - w not greater than 32
    /// - not supported for ReedSolVand
    Schedule,
    /// The schedule coding technique with cache.
    ///
    /// # Requires
    /// - same as `Schedule`
    /// - m must be 2
    /// - not supported for ReedSolVand
    ScheduleCache,
}

#[derive(Debug)]
enum TechInner {
    /// The matrix coding technique.
    ///
    /// # Requires
    /// - w must be in {8,16,32}
    Matrix(Matrix),
    /// The bit-matrix coding technique.
    ///
    /// # Requires
    /// - packet_size must be set
    /// - w not greater than 32
    BitMatrix(Matrix, i32),
    /// The schedule coding technique.
    ///
    /// # Requires
    /// - packet_size must be set
    /// - w not greater than 32
    Schedule(Schedule),
    /// # Requires
    /// - m must be 2
    ScheduleCache(ScheduleCache),
}

#[derive(Debug)]
struct Matrix {
    ptr: *mut c_int,
}

impl Matrix {
    /// Make a malloc box from a pointer from `malloc`.
    ///
    /// # Safety
    /// This function is unsafe because improper use may lead to memory problems. For example,
    /// a double-free may occur if the function is called twice on the same raw pointer.
    unsafe fn try_from_raw(ptr: *mut c_int) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        Some(Self { ptr })
    }

    fn as_ptr(&self) -> *mut c_int {
        self.ptr
    }

    fn as_mut_ptr(&mut self) -> *mut c_int {
        self.ptr
    }
}

impl Drop for Matrix {
    fn drop(&mut self) {
        unsafe {
            jerasure_sys::jerasure::jerasure_free_matrix(self.ptr);
        }
    }
}

#[derive(Debug)]
struct Schedule {
    bmat: Matrix,
    packet_size: i32,
    inner: *mut *mut c_int,
}

impl Drop for Schedule {
    fn drop(&mut self) {
        unsafe {
            jerasure_sys::jerasure::jerasure_free_schedule(self.inner);
        }
    }
}

#[derive(Debug)]
struct ScheduleCache {
    packet_size: i32,
    k: i32,
    m: i32,
    schedule: *mut *mut c_int,
    cache: *mut *mut *mut c_int,
}

impl Drop for ScheduleCache {
    fn drop(&mut self) {
        unsafe {
            jerasure_sys::jerasure::jerasure_free_schedule(self.schedule);
            jerasure_sys::jerasure::jerasure_free_schedule_cache(self.k, self.m, self.cache);
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// The `CodingMethod` is used to represent the coding method used to encode and decode the data.
///
/// Each method has its own matrix generation algorithm.
/// For more information, see the [jerasure documentation](https://github.com/tsuraan/Jerasure/blob/414c96ef2b9934953b6facb31d803d79b1dd1405/Manual.pdf)
pub enum CodingMethod {
    /// The Reed-Solomon Vandermonde coding method.
    ///
    /// # Requires
    /// - w must be in {8,16,32}
    /// - not supported for BitMatrix, Schedule, ScheduleCache
    ReedSolVand,
    /// The Cauchy coding method.
    ///
    /// # Requires
    /// - packet_size must be set to a multiple of the machine long size
    Cauchy,
    Liberation,
    Liber8tion,
    BlaumRoth,
}

/// The `ErasureCodeBuilder` is used to build the `ErasureCode` struct.
///
/// It is a builder pattern that allows you to set the parameters of the erasure code.
#[derive(Debug, Default, Clone)]
pub struct ErasureCodeBuilder {
    k: Option<i32>,
    m: Option<i32>,
    w: CodeWord,
    packet_size: Option<i32>,
    tech: Option<Technique>,
    coding_method: Option<CodingMethod>,
}

impl ErasureCodeBuilder {
    /// Create a new `ErasureCodeBuilder` instance.
    /// With default values:
    /// - `k` and `m` are not set
    /// - `w` is `CodeWord::W8`
    /// - `packet_size` is not set
    /// - `tech` is not set
    /// - `coding_method` is not set
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Set the number of data devices.
    pub fn k(mut self, k: NonZeroI32) -> Self {
        self.k = Some(k.get());
        self
    }

    /// Set the number of parity devices.
    pub fn m(mut self, m: NonZeroI32) -> Self {
        self.m = Some(m.get());
        self
    }

    /// Set the code word size.
    ///
    /// # Default
    /// - `CodeWord::W8`
    ///
    /// # Requires
    /// - $k + m <= 2^w$
    pub fn w(mut self, w: CodeWord) -> Self {
        self.w = w;
        self
    }

    /// Set the packet size.
    ///
    /// # Requires
    /// - `packet_size` must be a multiple of the machine long size
    pub fn packet_size(mut self, packet_size: NonZeroI32) -> Self {
        self.packet_size = Some(packet_size.get());
        self
    }

    /// Set the implementation technique.
    pub fn tech(mut self, tech: Technique) -> Self {
        self.tech = Some(tech);
        self
    }

    /// Set the coding method.
    pub fn coding_method(mut self, method: CodingMethod) -> Self {
        self.coding_method = Some(method);
        self
    }

    /// Build the `ErasureCode` struct.
    pub fn build(self) -> Result<ErasureCode, Error> {
        let k: i32 = self
            .k
            .ok_or_else(|| Error::invalid_arguments("k is required"))?;
        let m: i32 = self
            .m
            .ok_or_else(|| Error::invalid_arguments("m is required"))?;
        let tech = self
            .tech
            .ok_or_else(|| Error::invalid_arguments("tech is required"))?;
        let w = self.w;
        let coding_method = self
            .coding_method
            .ok_or_else(|| Error::invalid_arguments("coding_method is required"))?;
        if k <= 0 {
            return Err(Error::invalid_arguments("k must be greater than 0"));
        }
        if m <= 0 {
            return Err(Error::invalid_arguments("m must be greater than 0"));
        }
        if k + m > (1 << w.to_u8()) {
            return Err(Error::invalid_arguments(format!(
                "k + m must be less or equal than 2^w({})",
                1 << w.to_u8()
            )));
        }
        let mat = match coding_method {
            CodingMethod::ReedSolVand => self.reed_sol_vand_mat()?,
            CodingMethod::Cauchy => self.cauchy_mat()?,
            _ => unimplemented!("Liber8tion, BlaumRoth are not implemented yet"),
        };

        let tech = match tech {
            Technique::Matrix => {
                // w must be in {8,16,32}
                if matches!(w, CodeWord::Other(_)) {
                    return Err(Error::not_supported("w must be in {8,16,32}"));
                }
                TechInner::Matrix(mat)
            }
            Technique::BitMatrix => {
                if matches!(coding_method, CodingMethod::ReedSolVand) {
                    return Err(Error::not_supported(
                        "BitMatrix is not supported for ReedSolVand",
                    ));
                }
                let bmat = self.mat_to_bitmat(mat)?;
                TechInner::BitMatrix(bmat, self.check_packet_size()?)
            }
            Technique::Schedule => {
                if matches!(coding_method, CodingMethod::ReedSolVand) {
                    return Err(Error::not_supported(
                        "Schedule is not supported for ReedSolVand",
                    ));
                }
                let bmat = self.mat_to_bitmat(mat)?;
                let schedule = self.bmat_to_schedule(bmat)?;
                TechInner::Schedule(schedule)
            }
            Technique::ScheduleCache => {
                if matches!(coding_method, CodingMethod::ReedSolVand) {
                    return Err(Error::not_supported(
                        "ScheduleCache is not supported for ReedSolVand",
                    ));
                }
                if m != 2 {
                    return Err(Error::not_supported(
                        "ScheduleCache is only supported for m = 2",
                    ));
                }
                let bmat = self.mat_to_bitmat(mat)?;
                let schedule = self.bmat_toschedule_cache(bmat)?;
                TechInner::ScheduleCache(schedule)
            }
        };

        Ok(ErasureCode {
            tech,
            k,
            m,
            w,
            method: coding_method,
        })
    }
}

impl ErasureCodeBuilder {
    fn reed_sol_vand_mat(&self) -> Result<Matrix, Error> {
        let k = self.k.unwrap();
        let m = self.m.unwrap();
        let w = self.w;

        unsafe {
            Matrix::try_from_raw(jerasure_sys::jerasure::reed_sol_vandermonde_coding_matrix(
                k,
                m,
                w.as_cint(),
            ))
        }
        .ok_or_else(|| Error::other("Failed to create reed solomon vandermonde matrix"))
    }

    fn cauchy_mat(&self) -> Result<Matrix, Error> {
        let k = self
            .k
            .ok_or_else(|| Error::invalid_arguments("k is required"))?;
        let m = self
            .m
            .ok_or_else(|| Error::invalid_arguments("m is required"))?;
        let w = self.w;

        unsafe {
            Matrix::try_from_raw(jerasure_sys::jerasure::cauchy_good_general_coding_matrix(
                k,
                m,
                w.as_cint(),
            ))
        }
        .ok_or_else(|| Error::other("Failed to create cauchy matrix"))
    }

    fn mat_to_bitmat(&self, mut mat: Matrix) -> Result<Matrix, Error> {
        let k = self.k.unwrap();
        let m = self.m.unwrap();
        let w = self.w;

        unsafe {
            Matrix::try_from_raw(jerasure_sys::jerasure::jerasure_matrix_to_bitmatrix(
                k,
                m,
                w.as_cint(),
                mat.as_mut_ptr(),
            ))
        }
        .ok_or_else(|| Error::other("Failed to create bit matrix"))
    }

    fn bmat_to_schedule(&self, mut bmat: Matrix) -> Result<Schedule, Error> {
        let k = self.k.unwrap();
        let m = self.m.unwrap();
        let w = self.w;

        let p = unsafe {
            jerasure_sys::jerasure::jerasure_smart_bitmatrix_to_schedule(
                k,
                m,
                w.as_cint(),
                bmat.as_mut_ptr(),
            )
        };
        if p.is_null() {
            Err(Error::other("Failed to create schedule"))
        } else {
            Ok(Schedule {
                bmat,
                packet_size: self.check_packet_size()?,
                inner: p,
            })
        }
    }

    fn bmat_toschedule_cache(&self, mut bmat: Matrix) -> Result<ScheduleCache, Error> {
        let k = self.k.unwrap();
        let m = self.m.unwrap();
        let w = self.w;

        let schedule = unsafe {
            jerasure_sys::jerasure::jerasure_smart_bitmatrix_to_schedule(
                k,
                m,
                w.as_cint(),
                bmat.as_mut_ptr(),
            )
        };
        if schedule.is_null() {
            return Err(Error::other("Failed to create schedule"));
        }
        let cache = unsafe {
            jerasure_sys::jerasure::jerasure_generate_schedule_cache(
                k,
                m,
                w.as_cint(),
                bmat.as_mut_ptr(),
                1,
            )
        };
        if cache.is_null() {
            unsafe { jerasure_sys::jerasure::jerasure_free_schedule(schedule) };
            return Err(Error::other("Failed to create schedule cache"));
        }
        Ok(ScheduleCache {
            packet_size: self.check_packet_size()?,
            schedule,
            cache,
            k,
            m,
        })
    }

    fn check_packet_size(&self) -> Result<i32, Error> {
        if self.packet_size.is_none() {
            return Err(Error::invalid_arguments("packet_size is required"));
        }
        let packet_size = self.packet_size.unwrap();
        if packet_size <= 0 {
            return Err(Error::invalid_arguments(
                "packet_size must be greater than 0",
            ));
        }
        if packet_size % i32::try_from(crate::MACHINE_LONG_SIZE).unwrap() != 0 {
            return Err(Error::invalid_arguments(format!(
                "packet_size({packet_size}) must be a multiple of the machine long size({})",
                crate::MACHINE_LONG_SIZE as i32
            )));
        }

        Ok(packet_size)
    }
}

/// The `ErasureCode` struct is used to encode and decode data using erasure codes.
///
/// It is a wrapper around the Jerasure library, which provides efficient implementations
/// of various erasure coding techniques.
pub struct ErasureCode {
    k: i32,
    m: i32,
    w: CodeWord,
    tech: TechInner,
    method: CodingMethod,
}

impl ErasureCode {
    /// Return the number of data devices.
    pub fn k(&self) -> i32 {
        self.k
    }

    /// Return the number of parity devices.
    pub fn m(&self) -> i32 {
        self.m
    }

    /// Return the code word size.
    pub fn w(&self) -> CodeWord {
        self.w
    }

    /// Return the coding method.
    pub fn tech(&self) -> Technique {
        match &self.tech {
            TechInner::Matrix(_) => Technique::Matrix,
            TechInner::BitMatrix(_, _) => Technique::BitMatrix,
            TechInner::Schedule(_) => Technique::Schedule,
            TechInner::ScheduleCache(_) => Technique::ScheduleCache,
        }
    }

    fn _encode_parity<T: AsRef<[u8]>, U: AsMut<[u8]>>(
        &self,
        source: impl AsRef<[T]>,
        mut parity: U,
    ) -> Result<(), Error> {
        if source.as_ref().len() != self.k as usize {
            return Err(Error::invalid_arguments(
                "source must be the same length as k",
            ));
        }
        let parity = parity.as_mut();
        let src = source
            .as_ref()
            .iter()
            .map(|s| s.as_ref())
            .map(|s| {
                if s.len() % crate::MACHINE_LONG_SIZE != 0 {
                    return Err(Error::NotAligned(s.len()));
                }
                if s.len() != parity.len() {
                    Err(Error::invalid_arguments(
                        "source and parity must be the same length",
                    ))
                } else {
                    Ok(s)
                }
            })
            .map_ok(|s| s.as_ptr() as *mut ::std::ffi::c_char)
            .try_collect::<_, Vec<_>, Error>()?;
        unsafe {
            jerasure_sys::jerasure::jerasure_do_parity(
                self.k,
                src.as_ptr() as *mut *mut ::std::ffi::c_char,
                parity.as_mut_ptr() as *mut ::std::ffi::c_char,
                parity.len().try_into().unwrap(),
            );
        }
        Ok(())
    }

    /// Encode the data and generate coding parity.
    ///
    /// # Arguments
    /// * `data` - The data to encode, which must be a slice of `k` buffers of the same length.
    /// * `code` - The buffer to store the coding parity, which must be a slice of `m` buffers of the same length.
    ///
    /// # Requires
    /// * All the buffers must be aligned to the machine long size, and be the same length.
    pub fn encode<T: AsRef<[u8]>, U: AsMut<[u8]>>(
        &self,
        data: impl AsRef<[T]>,
        mut code: impl AsMut<[U]>,
    ) -> Result<(), Error> {
        self.check_encode_buffer(&data, &mut code)?;
        let len = data.as_ref().first().unwrap().as_ref().len();
        let src = data
            .as_ref()
            .iter()
            .map(|s| s.as_ref())
            .map(|s| s.as_ptr() as *mut ::std::ffi::c_char)
            .collect::<Vec<_>>();
        let parity = code
            .as_mut()
            .iter_mut()
            .map(|s| s.as_mut())
            .map(|s| s.as_mut_ptr() as *mut ::std::ffi::c_char)
            .collect::<Vec<_>>();
        let data_ptrs = src.as_ptr() as *mut *mut ::std::ffi::c_char;
        let coding_ptrs = parity.as_ptr() as *mut *mut ::std::ffi::c_char;
        match &self.tech {
            TechInner::Matrix(mat) => unsafe {
                jerasure_sys::jerasure::jerasure_matrix_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    mat.as_ptr(),
                    data_ptrs,
                    coding_ptrs,
                    len.try_into().unwrap(),
                );
            },
            TechInner::BitMatrix(bmat, packet_size) => unsafe {
                jerasure_sys::jerasure::jerasure_bitmatrix_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    bmat.as_ptr(),
                    data_ptrs,
                    coding_ptrs,
                    len.try_into().unwrap(),
                    *packet_size,
                );
            },
            TechInner::Schedule(schedule) => unsafe {
                jerasure_sys::jerasure::jerasure_schedule_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    schedule.inner,
                    data_ptrs,
                    coding_ptrs,
                    len.try_into().unwrap(),
                    schedule.packet_size,
                );
            },
            TechInner::ScheduleCache(schedule) => unsafe {
                jerasure_sys::jerasure::jerasure_schedule_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    schedule.schedule,
                    data_ptrs,
                    coding_ptrs,
                    len.try_into().unwrap(),
                    schedule.packet_size,
                );
            },
        }
        Ok(())
    }

    /// Decode the data and recover the erased data.
    ///
    /// # Arguments
    /// * `data` - The data devices, which must be a slice of `k` buffers of the same length.
    /// * `code` - The coding devices, which must be a slice of `m` buffers of the same length.
    /// * `erased` - The indices of the erased data devices, which must be a slice of integers.
    ///   The `k` data devices are indexed 0..k, and the `m` coding devices are indexed k..k+m.
    ///
    /// # Requires
    /// * All the buffers must be aligned to the machine long size, and be the same length.
    /// * The erased indices must be in the range of 0..k+m.
    /// * The number of erased indices must be less than or equal to `m`.
    /// * The erased indices must be unique.
    ///
    /// # Note
    /// The erased devices may not be recovered even if the number of erased devices is less than or equal to `m`.
    /// This is because the coding matrix may not be full rank with large `k` and `m`.
    /// In this case, the function will return an error.
    pub fn decode<T: AsMut<[u8]>>(
        &self,
        mut data: impl AsMut<[T]>,
        mut code: impl AsMut<[T]>,
        erased: &[i32],
    ) -> Result<(), Error> {
        use iter_tools::prelude::*;
        let erased: Result<Vec<_>, Error> = erased
            .iter()
            .sorted()
            .dedup()
            .map(|&i| {
                if 0 <= i && i < self.k + self.m {
                    Ok(i)
                } else {
                    Err(Error::invalid_arguments("erased index out of bounds"))
                }
            })
            .chain(std::iter::once(Ok(-1)))
            .try_collect();
        let erased = erased?;
        if erased.len() - 1 > self.m as usize {
            return Err(Error::too_many_erasure(erased.len() as i32 - 1, self.m));
        }
        self.check_decode_buffer(data.as_mut(), code.as_mut())?;

        let len = data.as_mut().first_mut().unwrap().as_mut().len();
        let src = data
            .as_mut()
            .iter_mut()
            .map(|s| s.as_mut())
            .map(|s| s.as_mut_ptr() as *mut ::std::ffi::c_char)
            .collect::<Vec<_>>();
        let parity = code
            .as_mut()
            .iter_mut()
            .map(|s| s.as_mut())
            .map(|s| s.as_mut_ptr() as *mut ::std::ffi::c_char)
            .collect::<Vec<_>>();

        let row_k_ones = matches!(self.method, CodingMethod::ReedSolVand)
            .then_some(1)
            .unwrap_or(0);
        let erasures_ptr = erased.as_ptr() as *mut i32;
        let data_ptrs = src.as_ptr() as *mut *mut ::std::ffi::c_char;
        let coding_ptrs = parity.as_ptr() as *mut *mut ::std::ffi::c_char;
        match &self.tech {
            TechInner::Matrix(mat) => {
                let ret = unsafe {
                    jerasure_sys::jerasure::jerasure_matrix_decode(
                        self.k,
                        self.m,
                        self.w.as_cint(),
                        mat.as_ptr(),
                        row_k_ones,
                        erasures_ptr,
                        data_ptrs,
                        coding_ptrs,
                        len.try_into().unwrap(),
                    )
                };
                if ret != 0 {
                    return Err(Error::other("Failed to decode"));
                }
            }
            TechInner::BitMatrix(malloc_box, packet_size) => {
                let ret = unsafe {
                    jerasure_sys::jerasure::jerasure_bitmatrix_decode(
                        self.k,
                        self.m,
                        self.w.as_cint(),
                        malloc_box.as_ptr(),
                        row_k_ones,
                        erasures_ptr,
                        data_ptrs,
                        coding_ptrs,
                        len.try_into().unwrap(),
                        *packet_size,
                    )
                };
                if ret != 0 {
                    return Err(Error::other("Failed to decode"));
                }
            }
            TechInner::Schedule(schedule) => {
                let ret = unsafe {
                    jerasure_sys::jerasure::jerasure_schedule_decode_lazy(
                        self.k,
                        self.m,
                        self.w.as_cint(),
                        schedule.bmat.as_ptr(),
                        erased.as_ptr() as *mut i32,
                        src.as_ptr() as *mut *mut ::std::ffi::c_char,
                        parity.as_ptr() as *mut *mut ::std::ffi::c_char,
                        len.try_into().unwrap(),
                        schedule.packet_size,
                        1,
                    )
                };
                if ret != 0 {
                    return Err(Error::other("Failed to decode"));
                }
            }
            TechInner::ScheduleCache(schedule) => {
                let ret = unsafe {
                    jerasure_sys::jerasure::jerasure_schedule_decode_cache(
                        self.k,
                        self.m,
                        self.w.as_cint(),
                        schedule.cache,
                        erasures_ptr,
                        data_ptrs,
                        coding_ptrs,
                        len.try_into().unwrap(),
                        schedule.packet_size,
                    )
                };
                if ret != 0 {
                    return Err(Error::other("Failed to decode"));
                }
            }
        }

        Ok(())
    }
}

impl ErasureCode {
    fn check_encode_buffer<T: AsRef<[u8]>, U: AsMut<[u8]>>(
        &self,
        source: impl AsRef<[T]>,
        mut parity: impl AsMut<[U]>,
    ) -> Result<(), Error> {
        let source = source.as_ref();
        let parity = parity.as_mut();
        if source.len() != self.k as usize {
            return Err(Error::invalid_arguments(format!(
                "source must have k({}) elements",
                self.k
            )));
        }
        if parity.len() != self.m as usize {
            return Err(Error::invalid_arguments(format!(
                "parity must have m({}) elements",
                self.m,
            )));
        }
        let len = source.first().unwrap().as_ref().len();
        for s in source.as_ref().iter() {
            let s = s.as_ref();
            if s.len() % crate::MACHINE_LONG_SIZE != 0 {
                return Err(Error::NotAligned(s.as_ref().len()));
            }
            if s.len() != len {
                return Err(Error::invalid_arguments(
                    "source and parity must be the same length",
                ));
            }
        }
        for p in parity {
            let p = p.as_mut();
            if p.len() % crate::MACHINE_LONG_SIZE != 0 {
                return Err(Error::NotAligned(p.len()));
            }
            if p.len() != len {
                return Err(Error::invalid_arguments(
                    "source and parity must be the same length",
                ));
            }
        }
        Ok(())
    }

    fn check_decode_buffer<T: AsMut<[u8]>, U: AsMut<[u8]>>(
        &self,
        mut source: impl AsMut<[T]>,
        mut parity: impl AsMut<[U]>,
    ) -> Result<(), Error> {
        let source = source.as_mut();
        let parity = parity.as_mut();
        if source.len() != self.k as usize {
            return Err(Error::invalid_arguments(format!(
                "source must have k({}) elements",
                self.k
            )));
        }
        if parity.len() != self.m as usize {
            return Err(Error::invalid_arguments(format!(
                "parity must have m({}) elements",
                self.m,
            )));
        }
        let len = source.first_mut().unwrap().as_mut().len();
        for s in source.as_mut().iter_mut() {
            let s = s.as_mut();
            if s.len() % crate::MACHINE_LONG_SIZE != 0 {
                return Err(Error::NotAligned(s.as_ref().len()));
            }
            if s.len() != len {
                return Err(Error::invalid_arguments(
                    "source and parity must be the same length",
                ));
            }
        }
        for p in parity {
            let p = p.as_mut();
            if p.len() % crate::MACHINE_LONG_SIZE != 0 {
                return Err(Error::NotAligned(p.len()));
            }
            if p.len() != len {
                return Err(Error::invalid_arguments(
                    "source and parity must be the same length",
                ));
            }
        }
        Ok(())
    }
}
