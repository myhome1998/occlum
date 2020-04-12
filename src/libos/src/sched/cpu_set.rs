//! A CpuSet is a bit mask used to represent a set of CPU cores.
//!
//! The number of bits contained in a CpuSet equals to the number of CPU cores
//! on the current platform. The bits in a CpuSet can be accessible via indexes
//! or iterators.
//!
//! The meaning of the i-th bits in a CpuSet `cpu_set`is as follows:
//! * If `cpu_set[i] == true`, then the i-th CPU core belongs to the set;
//! * Otherwise, the i-th CPU core is not in the set.

use bitvec::prelude::*;
use std::ops::Index;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CpuSet {
    bits: BitBox<Local, u8>,
}

impl CpuSet {
    /// Returns the length of a CPU set in bytes.
    pub fn len() -> usize {
        align_up(Self::ncores(), 8) / 8
    }

    /// Returns the number CPU of cores in a CPU set.
    pub fn ncores() -> usize {
        *NCORES
    }

    /// Create a CpuSet that consists of all of the CPU cores.
    pub fn new_full() -> Self {
        let mut bits = bitbox![Local, u8; 1; Self::len() * 8];
        Self::clear_unused(&mut bits);
        Self { bits }
    }

    /// Create a CpuSet that consists of none of the CPU cores.
    pub fn new_empty() -> Self {
        let bits = bitbox![Local, u8; 0; Self::len() * 8];
        Self { bits }
    }

    /// Returns if the CpuSet has no CPU cores.
    pub fn full(&self) -> bool {
        self.bits.count_ones() == Self::ncores()
    }

    /// Returns if the CpuSet has no CPU cores.
    pub fn empty(&self) -> bool {
        self.bits.count_ones() == 0
    }

    /// Create a CpuSet from bits given in a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() < Self::len() {
            return_errno!(EINVAL, "slice is not long enough");
        }
        let slice = &slice[..Self::len()];
        let mut bits = BitBox::from_slice(slice);
        Self::clear_unused(&mut bits);

        Ok(Self { bits })
    }

    /// Returns the underlying byte slice.
    ///
    /// The last, unused bits in the byte slice are guaranteed to be zero.
    pub fn as_slice(&self) -> &[u8] {
        self.bits.as_slice()
    }

    /// Returns an iterator that allows accessing the underlying bits.
    pub fn iter(&self) -> Iter {
        self.bits.iter()
    }

    /// Returns an iterator that allows modifying the underlying bits.
    pub fn iter_mut(&mut self) -> IterMut {
        self.bits.iter_mut()
    }

    fn clear_unused(bits: &mut BitSlice<Local, u8>) {
        let unused_bits = &mut bits[Self::ncores()..(Self::len() * 8)];
        for mut bit in unused_bits {
            *bit = false;
        }
    }
}

pub type Iter<'a> = bitvec::slice::Iter<'a, Local, u8>;
pub type IterMut<'a> = bitvec::slice::IterMut<'a, Local, u8>;

impl Index<usize> for CpuSet {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        assert!(index < Self::ncores());
        &self.bits[index]
    }
}

lazy_static! {
    /// The number of all CPU cores on the platform
    static ref NCORES: usize = {
        extern "C" {
            fn occlum_ocall_ncores(ret: *mut i32) -> sgx_status_t;
        }
        unsafe {
            let mut ncores = 0;
            let status = occlum_ocall_ncores(&mut ncores);
            assert!(
                status == sgx_status_t::SGX_SUCCESS &&
                // Ncores == 0 is meaningless
                0 < ncores &&
                // A reasonble upper limit for the foreseeable future
                ncores <= 1024
            );
            ncores as usize
        }
    };
}
