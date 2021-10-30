#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct BitVec {
    bytes: Vec<u8>,
    len: usize,
}

const BITS: usize = 8;

impl BitVec {
    pub fn with_size(len: usize) -> Self {
        let num_bytes = (len + BITS - 1) / BITS;
        BitVec {
            bytes: vec![0; num_bytes],
            len,
        }
    }

    pub fn set(&mut self, idx: usize) {
        if idx >= self.len {
            panic!(
                "index {} out of bounds for BitVec of length {}",
                idx, self.len
            );
        }
        let byte = idx / BITS;
        let rem = BITS - 1 - (idx % BITS);

        self.bytes[byte] |= 1 << rem;
    }

    pub fn to_vec(&self) -> Vec<bool> {
        let mut vec = vec![false; self.len];
        for (i, byte) in self.bytes.iter().enumerate() {
            for bit in 0..BITS {
                if byte & (1 << (BITS - 1 - bit)) != 0 {
                    vec[i * BITS + bit] = true;
                }
            }
        }
        vec
    }
}
