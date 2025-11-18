use std::ops::{Add, AddAssign};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Simple 192-bit unsigned integer implemented as 3 little-endian u64 limbs.
/// Provides the small API used by the consensus core (From<u64>, AddAssign, Add, to_bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Uint192([u64; 3]);

/// Empty MuHash constant representing zero in MuHash context
pub const EMPTY_MUHASH: Uint192 = Uint192([0; 3]);

impl From<u64> for Uint192 {
    fn from(v: u64) -> Self {
        Self([v, 0, 0])
    }
}

impl Uint192 {
    /// Returns little-endian bytes (24 bytes)
    pub fn to_bytes(&self) -> [u8; 24] {
        let mut out = [0u8; 24];
        out[0..8].copy_from_slice(&self.0[0].to_le_bytes());
        out[8..16].copy_from_slice(&self.0[1].to_le_bytes());
        out[16..24].copy_from_slice(&self.0[2].to_le_bytes());
        out
    }
}

impl AddAssign for Uint192 {
    fn add_assign(&mut self, rhs: Self) {
        let (r0, carry0) = self.0[0].overflowing_add(rhs.0[0]);
        let (r1_tmp, carry1a) = self.0[1].overflowing_add(rhs.0[1]);
        let (r1, carry1b) = r1_tmp.overflowing_add(if carry0 { 1 } else { 0 });
        let carry1 = carry1a || carry1b;
        let (r2_tmp, _carry2a) = self.0[2].overflowing_add(rhs.0[2]);
        let (r2, _carry2b) = r2_tmp.overflowing_add(if carry1 { 1 } else { 0 });
        self.0 = [r0, r1, r2];
    }
}

impl Add for Uint192 {
    type Output = Uint192;
    fn add(self, rhs: Self) -> Self::Output {
        let mut r = self;
        r += rhs;
        r
    }
}

impl fmt::Display for Uint192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert to hex string for display
        let bytes = self.to_bytes();
        for byte in bytes.iter().rev() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Uint192;

    #[test]
    fn add_assign_no_overflow() {
        let mut a = Uint192::from(1u64);
        let b = Uint192::from(2u64);
        a += b;
        assert_eq!(a.to_bytes()[0..8], 3u64.to_le_bytes());
    }

    #[test]
    fn to_bytes_length() {
        let a = Uint192::from(0x11223344u64);
        let bytes = a.to_bytes();
        assert_eq!(bytes.len(), 24);
        assert_eq!(&bytes[0..8], &0x11223344u64.to_le_bytes());
    }
}
