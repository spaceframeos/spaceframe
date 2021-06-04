use std::cmp;

use bitvec::prelude::*;
use chacha20::{
    cipher::{NewCipher, StreamCipher, StreamCipherSeek},
    ChaCha8, Key, Nonce,
};

use crate::{Bits, BitsSlice, constants::{PARAM_EXT, STATE_SIZE_BITS}, utils::{divmod, from_bits}};

#[derive(Debug, Clone)]
pub struct F1Calculator {
    plot_seed: Vec<u8>,
    k: usize,
}

impl F1Calculator {
    pub fn new(k: usize, plot_seed: &[u8]) -> Self {
        F1Calculator {
            k,
            plot_seed: plot_seed.to_vec(),
        }
    }

    pub fn calculate_f1(&self, x: &BitsSlice) -> Bits {
        assert_eq!(x.len(), self.k, "x must be k bits");

        let (q, r) = divmod(from_bits(x) * self.k as u64, STATE_SIZE_BITS as u64);

        let key = Key::from_slice(self.plot_seed.as_slice());
        let nonce = Nonce::from_slice(b"000000000000");

        let mut cipher = ChaCha8::new(&key, &nonce);
        cipher.seek(q);

        let mut ciphertext0 = [0; STATE_SIZE_BITS / 8];
        cipher.apply_keystream(&mut ciphertext0);

        let mut result = if r + self.k as u64 > STATE_SIZE_BITS as u64 {
            // Span two state of 512 bits
            cipher.seek(q + 1);
            let mut ciphertext1 = [0; STATE_SIZE_BITS / 8];
            cipher.apply_keystream(&mut ciphertext1);
            let mut result = ciphertext0.view_bits()[r as usize..].to_bitvec();
            result.extend_from_bitslice(
                &ciphertext1.view_bits::<Msb0>()
                    [0..(r + self.k as u64 - STATE_SIZE_BITS as u64) as usize],
            );
            result
        } else {
            let result = ciphertext0.view_bits().to_bitvec();
            result[r as usize..r as usize + self.k].to_bitvec()
        };

        let extension = &x[..cmp::min(PARAM_EXT, x.len()) as usize];
        result.extend_from_bitslice(extension);
        if x.len() < PARAM_EXT {
            result.append(&mut bitvec![0; PARAM_EXT - x.len()]);
        }
        result
    }
}
