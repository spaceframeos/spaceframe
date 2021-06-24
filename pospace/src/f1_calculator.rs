use bitvec::prelude::*;
use chacha20::{
    cipher::{NewCipher, StreamCipher, StreamCipherSeek},
    ChaCha8, Key, Nonce,
};

use crate::{Bits, BitsSlice, bits::BitsWrapper, constants::{PARAM_EXT, STATE_SIZE_BITS}, utils::{divmod}};

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

    pub fn calculate_f1(&self, x: &BitsWrapper) -> Bits {
        assert_eq!(x.bits.len(), self.k, "x must be k bits");
        assert!(x.bits.len() >= PARAM_EXT, "x must greater or equal to {} bits", PARAM_EXT);

        let (q, r) = divmod(x.value * self.k as u64, STATE_SIZE_BITS as u64);

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

        let extension = &x.bits[..PARAM_EXT];
        result.extend_from_bitslice(extension);
        result
    }
}
