use ::std::os::raw::c_int;
use std::num::NonZeroI32;

use crate::{CodeWord, Error, MallocBox};

use iter_tools::Itertools;

#[derive(Debug, Clone, Copy, Default)]
pub enum Technique {
    #[default]
    Matrix,
    BitMatrix,
    Schedule,
}

#[derive(Debug)]
enum TechInner {
    /// The matrix technique.
    ///
    /// # Requires
    /// - w must be in {8,16,32}
    Matrix(MallocBox<c_int>),
    BitMatrix(MallocBox<c_int>, i32),
    Schedule,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum CodingMethod {
    #[default]
    ReedSolVand,
    Caychy,
    CauchyGood,
    Liberation,
    Liber8tion,
    BlaumRoth,
}

#[derive(Debug, Default, Clone)]
pub struct ErasureCodeBuilder {
    k: Option<i32>,
    m: Option<i32>,
    w: CodeWord,
    packet_size: Option<i32>,
    tech: Option<Technique>,
    coding_method: CodingMethod,
}

impl ErasureCodeBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn k(mut self, k: NonZeroI32) -> Self {
        self.k = Some(k.get());
        self
    }

    pub fn m(mut self, m: NonZeroI32) -> Self {
        self.m = Some(m.get());
        self
    }

    /// Set the code word size.
    ///
    /// # Default
    /// - `CodeWord::W8`
    pub fn w(mut self, w: CodeWord) -> Self {
        self.w = w;
        self
    }

    pub fn packet_size(mut self, packet_size: NonZeroI32) -> Self {
        self.packet_size = Some(packet_size.get());
        self
    }

    pub fn tech(mut self, tech: Technique) -> Self {
        self.tech = Some(tech);
        self
    }

    pub fn coding_method(mut self, method: CodingMethod) -> Self {
        self.coding_method = method;
        self
    }

    pub fn build(self) -> Result<ErasureCode, Error> {
        let k: i32 = self.k.unwrap();
        let m: i32 = self.m.unwrap();
        let w = self.w;
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
        let mut mat = match self.coding_method {
            CodingMethod::ReedSolVand => self.reed_sol_vand_mat()?,
            CodingMethod::Caychy => todo!(),
            CodingMethod::CauchyGood => todo!(),
            CodingMethod::Liberation => todo!(),
            CodingMethod::Liber8tion => todo!(),
            CodingMethod::BlaumRoth => todo!(),
        };

        let tech = match self.tech {
            Some(tech) => match tech {
                Technique::Matrix => {
                    // w must be in {8,16,32}
                    if matches!(w, CodeWord::Other(_)) {
                        return Err(Error::not_supported("w must be in {8,16,32}"));
                    }
                    TechInner::Matrix(mat)
                }
                Technique::BitMatrix => {
                    if matches!(self.coding_method, CodingMethod::ReedSolVand) {
                        return Err(Error::not_supported(
                            "BitMatrix is not supported for ReedSolVand",
                        ));
                    }
                    let bmat = unsafe {
                        MallocBox::try_from_raw(
                            jerasure_sys::jerasure::jerasure_matrix_to_bitmatrix(
                                k,
                                m,
                                w.as_cint(),
                                mat.as_mut_ptr(),
                            ),
                        )
                    }
                    .ok_or_else(|| Error::other("Failed to create bit matrix"))?;
                    TechInner::BitMatrix(
                        bmat,
                        self.packet_size
                            .ok_or_else(|| Error::invalid_arguments("packet_size is required"))?,
                    )
                }
                Technique::Schedule => {
                    if matches!(self.coding_method, CodingMethod::ReedSolVand) {
                        return Err(Error::not_supported(
                            "Schedule is not supported for ReedSolVand",
                        ));
                    }
                    TechInner::Schedule
                }
            },
            None => return Err(Error::other("tech is required")),
        };

        Ok(ErasureCode {
            tech,
            k,
            m,
            w,
            method: self.coding_method,
        })
    }
}

impl ErasureCodeBuilder {
    fn reed_sol_vand_mat(&self) -> Result<MallocBox<c_int>, Error> {
        let k = self
            .k
            .ok_or_else(|| Error::invalid_arguments("k is required"))?;
        let m = self
            .m
            .ok_or_else(|| Error::invalid_arguments("m is required"))?;
        let w = self.w;

        unsafe {
            MallocBox::try_from_raw(jerasure_sys::jerasure::reed_sol_vandermonde_coding_matrix(
                k,
                m,
                w.as_cint(),
            ))
        }
        .ok_or_else(|| Error::other("Failed to create reed solomon vandermonde matrix"))
    }
}

pub struct ErasureCode {
    k: i32,
    m: i32,
    w: CodeWord,
    tech: TechInner,
    method: CodingMethod,
}

impl ErasureCode {
    pub fn k(&self) -> i32 {
        self.k
    }

    pub fn m(&self) -> i32 {
        self.m
    }

    pub fn w(&self) -> CodeWord {
        self.w
    }

    pub fn tech(&self) -> Technique {
        match &self.tech {
            TechInner::Matrix(_) => Technique::Matrix,
            TechInner::BitMatrix(_, _) => Technique::BitMatrix,
            TechInner::Schedule => Technique::Schedule,
        }
    }

    pub fn encode_parity<'a, T: AsRef<[u8]> + 'a, U: AsMut<[u8]> + 'a>(
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

    pub fn encode<'a, T: AsRef<[u8]> + 'a, U: AsMut<[u8]> + 'a>(
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
        match &self.tech {
            TechInner::Matrix(mat) => unsafe {
                jerasure_sys::jerasure::jerasure_matrix_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    mat.as_ptr(),
                    src.as_ptr() as *mut *mut ::std::ffi::c_char,
                    parity.as_ptr() as *mut *mut ::std::ffi::c_char,
                    len.try_into().unwrap(),
                );
            },
            TechInner::BitMatrix(bmat, packet_size) => unsafe {
                jerasure_sys::jerasure::jerasure_bitmatrix_encode(
                    self.k,
                    self.m,
                    self.w.as_cint(),
                    bmat.as_ptr(),
                    src.as_ptr() as *mut *mut ::std::ffi::c_char,
                    parity.as_ptr() as *mut *mut ::std::ffi::c_char,
                    len.try_into().unwrap(),
                    *packet_size,
                );
            },
            TechInner::Schedule => todo!(),
        }
        Ok(())
    }

    pub fn decode<T: AsMut<[u8]>>(
        &self,
        mut data: impl AsMut<[T]>,
        mut code: impl AsMut<[T]>,
        erased: &[i32],
    ) -> Result<(), Error> {
        use iter_tools::prelude::*;
        let erased: Result<Vec<_>, Error> = erased
            .iter()
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
            return Err(Error::too_many_erased(erased.len() as i32 - 1, self.m));
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
        match &self.tech {
            TechInner::Matrix(mat) => {
                let ret = unsafe {
                    jerasure_sys::jerasure::jerasure_matrix_decode(
                        self.k,
                        self.m,
                        self.w.as_cint(),
                        mat.as_ptr(),
                        row_k_ones,
                        erased.as_ptr() as *mut i32,
                        src.as_ptr() as *mut *mut ::std::ffi::c_char,
                        parity.as_ptr() as *mut *mut ::std::ffi::c_char,
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
                        erased.as_ptr() as *mut i32,
                        src.as_ptr() as *mut *mut ::std::ffi::c_char,
                        parity.as_ptr() as *mut *mut ::std::ffi::c_char,
                        len.try_into().unwrap(),
                        *packet_size,
                    )
                };
                if ret != 0 {
                    return Err(Error::other("Failed to decode"));
                }
            }
            TechInner::Schedule => todo!(),
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
