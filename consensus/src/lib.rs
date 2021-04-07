
use chacha20::{ChaCha8, Key, Nonce};
use chacha20::cipher::{NewStreamCipher, SyncStreamCipher, SyncStreamCipherSeek};
use bitvec::{prelude::*};
use std::cmp;


const param_EXT: u64 = 6;
const k: u64 = 10;
const fsize: u64 = param_EXT + k;
const param_M: u64 = 1 << param_EXT;
const param_B: u64 = 119;
const param_C: u64 = 127;
const param_BC: u64 = param_B * param_C;
const param_c1: u64 = 1000;
const param_c2: u64 = 1000;
const blocksize_bits: u64 = 512;

type Bits = BitVec<Msb0, u8>;
type BitsSlice = BitSlice<Msb0, u8>;

fn bucket_id(x: u64) -> u64 {
    (x as f64 / param_BC as f64).floor() as u64
}

fn divmod(x: u64, m: u64) -> (u64, u64) {
    (x.div_euclid(m), x.rem_euclid(m))
}

fn b_id(x: u64) -> u64 {
    divmod(x % param_BC, param_C).0
}

fn c_id(x: u64) -> u64 {
    divmod(x % param_BC, param_C).1
}

fn colla_size(t: u64) -> Option<u64> {
    match t {
        2 => Some(1),
        3 | 7 => Some(2),
        6 => Some(3),
        4 | 5 => Some(4),
        _ => None
    }
}

/// Matching function
fn M(l: u64, r: u64) -> bool {
    if bucket_id(l) + 1 != bucket_id(r) {
        return false;
    }

    for m in 0..param_M {
        if (b_id(r) as i64 - b_id(l) as i64).rem_euclid(param_M as i64) == m as i64 {
            if (c_id(r) as i64 - c_id(l) as i64).rem_euclid(param_M as i64) == ((2 * m + (bucket_id(l) % 2)).pow(2) % param_C) as i64 {
                return true;
            }
        }
    }

    return false;
}

fn bits_slice(x: u64, start_index: u64, end_index: u64) -> u64 {
    let z: u64 = (x as f64).log2() as u64 + 1;
    (x >> (z - end_index)) % (1 << (end_index - start_index))
}

fn calculate_f1(x: &BitSlice<Msb0, u8>) -> BitVec<Msb0, u8> {
    assert!(x.len() == param_M as usize, "x must be 64 bits");

    let (q, r) = divmod(x.load_be::<u64>() * k, blocksize_bits);

    let key = Key::from_slice(b"an example plot seed key of 32b.");
    let nonce = Nonce::from_slice(b"000000000000");

    let mut cipher = ChaCha8::new(&key, &nonce);

    cipher.seek(q);
    let mut ciphertext0 = [0 as u8; (blocksize_bits / 8) as usize];
    cipher.apply_keystream(&mut ciphertext0);

    println!("k={}, bits_before_x={}, counter_bit={}", k, r, q);

    let mut result = if r + k > blocksize_bits {
        // Span two blocks
        cipher.seek(q + 1);
        let mut ciphertext1 = [0 as u8; (blocksize_bits / 8) as usize];
        cipher.apply_keystream(&mut ciphertext1);

        let mut result = ciphertext0.view_bits()[r as usize..].to_bitvec();
        result.extend_from_bitslice(&ciphertext1.view_bits::<Msb0>()[0..(r + k - blocksize_bits) as usize]);
        result
    } else {
        let result = ciphertext0.view_bits::<Msb0>().to_bitvec();
        result[r as usize .. (r + k) as usize].to_bitvec()
    };

    result.extend_from_bitslice(&x[..6 as usize]);
    println!("{}", result);
    result
}

fn calculate_bucket(x: &BitSlice<Msb0, u8>) -> (BitVec<Msb0, u8>, u64) {
    (calculate_f1(x), x.load_be::<u64>())
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;
    use super::*;

    #[test]
    fn test_bits() {
        let bytes = (13271 as u32).to_ne_bytes();
        let bit_slice = bytes.view_bits::<Lsb0>();
        println!("{}", bit_slice);
    }

    #[test]
    fn test_chacha() {
        let mut data = [1, 2, 3, 4, 5, 6, 7];

        let key = Key::from_slice(b"an example very very secret key.");
        let nonce = Nonce::from_slice(b"secret nonce");

        let mut cipher = ChaCha8::new(&key, &nonce);

        cipher.apply_keystream(&mut data);
        assert_eq!(data, [29, 96, 133, 82, 113, 15, 8]);
    }

    #[test]
    fn test_f1() {
        // for x in 0..(2 as u64).pow(k as u32) {
            // calculate_f1(&x.to_be_bytes().view_bits());
        // }
        let x: u64 = 13271;
        let fx = calculate_f1(&x.to_be_bytes().view_bits());
        println!("Len: {}, must be {}", fx.len(), (2 as u64).pow(fsize as u32));
    }

    #[test]
    fn test_bit_slice() {
        let x = 0b100100;
        assert_eq!(0b010, bits_slice(x, 2, 5));
    }
}
