use std::hash::Hasher;

use twox_hash::XxHash64;
pub struct CustomHash(twox_hash::XxHash64);

impl CustomHash {
    pub fn new() -> Self {
        let h = XxHash64::with_seed(1337);
        Self(h)
    }
    pub fn with_seed(seed: u64) -> Self {
        let h = XxHash64::with_seed(seed);
        Self(h)
    }
}

impl Hasher for CustomHash {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
    // yeah, lets not, it messes with hashing of BTree on different platforms
    fn write_length_prefix(&mut self, _len: usize) {
        // self.write_u64(len as u64);
    }
}
