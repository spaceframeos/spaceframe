use std::cmp;

use bitvec::prelude::*;
use chacha20::{
    cipher::{NewCipher, StreamCipher, StreamCipherSeek},
    ChaCha8, Key, Nonce,
};

use crate::{
    constants::{BLOCKSIZE_BITS, PARAM_EXT},
    utils::divmod,
    Bits, BitsSlice,
};

#[derive(Debug)]
pub struct F1Calculator {
    plot_seed: Vec<u8>,
    k: usize,
}

impl F1Calculator {

    pub fn new(k: usize, plot_seed: &[u8]) -> Self {
        F1Calculator {
            k,
            plot_seed: plot_seed.to_vec()
        }
    }

    pub fn calculate_f1(&self, x: &BitsSlice) -> Bits {
        assert_eq!(x.len(), self.k, "x must be k bits");
    
        let (q, r) = divmod(x.load_be::<u64>() * self.k as u64, BLOCKSIZE_BITS);
    
        let key = Key::from_slice(self.plot_seed.as_slice());
        let nonce = Nonce::from_slice(b"000000000000");
    
        let mut cipher = ChaCha8::new(&key, &nonce);
    
        cipher.seek(q);
        let mut ciphertext0 = [0; (BLOCKSIZE_BITS / 8) as usize];
        cipher.apply_keystream(&mut ciphertext0);
    
        // println!("k={}, bits_before_x={}, counter_bit={}", k, r, q);
    
        let mut result = if r + self.k as u64 > BLOCKSIZE_BITS {
            // Span two blocks
            cipher.seek(q + 1);
            let mut ciphertext1 = [0; (BLOCKSIZE_BITS / 8) as usize];
            cipher.apply_keystream(&mut ciphertext1);
    
            let mut result = ciphertext0.view_bits()[r as usize..].to_bitvec();
            result.extend_from_bitslice(
                &ciphertext1.view_bits::<Msb0>()[0..(r + self.k as u64 - BLOCKSIZE_BITS) as usize],
            );
            result
        } else {
            let result = ciphertext0.view_bits::<Msb0>().to_bitvec();
            result[r as usize..r as usize + self.k].to_bitvec()
        };
    
        result.extend_from_bitslice(&x[..cmp::min(PARAM_EXT, x.len()) as usize]);
        if x.len() < PARAM_EXT {
            result.append(&mut bitvec![0; PARAM_EXT - x.len()]);
        }
        // println!("x = {}, f1(x) = {}", x, result);
        result
    }

}
