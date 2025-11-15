use crypto_hashes::Hash;
use std::num::Wrapping;

pub struct XoShiRo256PlusPlus {
    s0: Wrapping<u64>,
    s1: Wrapping<u64>,
    s2: Wrapping<u64>,
    s3: Wrapping<u64>,
}

impl XoShiRo256PlusPlus {
    #[inline(always)]
    pub fn new(hash: Hash) -> Self {
        // Convert 32-byte Hash into four little-endian u64 words
        let bytes = hash.as_bytes();
        let mut parts = [0u64; 4];
        for i in 0..4 {
            let start = i * 8;
            parts[i] = u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap());
        }
        Self { s0: Wrapping(parts[0]), s1: Wrapping(parts[1]), s2: Wrapping(parts[2]), s3: Wrapping(parts[3]) }
    }

    #[inline(always)]
    pub fn u64(&mut self) -> u64 {
        let res = self.s0 + Wrapping((self.s0 + self.s3).0.rotate_left(23));
        let t = self.s1 << 17;
        self.s2 ^= self.s0;
        self.s3 ^= self.s1;
        self.s1 ^= self.s2;
        self.s0 ^= self.s3;

        self.s2 ^= t;
        self.s3 = Wrapping(self.s3.0.rotate_left(45));

        res.0
    }
}