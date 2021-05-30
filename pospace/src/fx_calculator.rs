use bitvec::prelude::*;

use crate::{constants::PARAM_EXT, f1_calculator::F1Calculator, Bits, BitsSlice};

#[derive(Debug)]
pub struct FXCalculator {
    k: usize,
    f_size: usize,
    f1_calculator: F1Calculator,
}

impl FXCalculator {
    pub fn new(k: usize, f1_calculator: F1Calculator) -> Self {
        FXCalculator {
            k,
            f1_calculator,
            f_size: k + PARAM_EXT,
        }
    }

    pub fn calculate_fn(&self, input: &[&BitsSlice]) -> Bits {
        let half_len = input.len() / 2;
        if input.len() == 2 {
            calculate_blake_hash(
                &self.collate_n(&input[..half_len]).unwrap(),
                &self.collate_n(&input[half_len..]).unwrap(),
                &self.f1_calculator.calculate_f1(&input[0]),
            )[..self.f_size]
                .to_bitvec()
        } else {
            calculate_blake_hash(
                &self.collate_n(&input[..half_len]).unwrap(),
                &self.collate_n(&input[half_len..]).unwrap(),
                &self.calculate_fn(&input[..half_len]),
            )[..self.f_size]
                .to_bitvec()
        }
    }

    pub fn collate_n(&self, input: &[&BitsSlice]) -> Result<Bits, ()> {
        match input.len() {
            1 => {
                return Ok(input[0].to_bitvec());
            }
            2 => {
                let mut out = input[0].to_bitvec();
                out.extend_from_bitslice(input[1]);
                return Ok(out);
            }
            4 => {
                let mut out = input[0].to_bitvec();
                out.extend_from_bitslice(input[1]);
                out.extend_from_bitslice(input[2]);
                out.extend_from_bitslice(input[3]);
                return Ok(out);
            }
            8 => {
                return Ok(calculate_blake_hash(
                    &self.collate_n(&input[0..4]).unwrap(),
                    &self.collate_n(&input[4..8]).unwrap(),
                    &self.calculate_fn(&input[0..4]),
                )[self.f_size..self.f_size + 4 * self.k]
                    .to_bitvec());
            }
            16 => {
                return Ok(calculate_blake_hash(
                    &self.collate_n(&input[0..8]).unwrap(),
                    &self.collate_n(&input[8..16]).unwrap(),
                    &self.calculate_fn(&input[0..8]),
                )[self.f_size..self.f_size + 3 * self.k]
                .to_bitvec());
            }
            32 => {
                return Ok(calculate_blake_hash(
                    &self.collate_n(&input[0..16]).unwrap(),
                    &self.collate_n(&input[16..32]).unwrap(),
                    &self.calculate_fn(&input[0..16]),
                )[self.f_size..self.f_size + 2 * self.k]
                .to_bitvec());
            }
            _ => {
                return Err(());
            }
        }
    }
}

pub fn calculate_blake_hash(y: &BitsSlice, l: &BitsSlice, r: &BitsSlice) -> Bits {
    let mut hasher = blake3::Hasher::new();
    hasher.update(y.as_raw_slice());
    hasher.update(l.as_raw_slice());
    hasher.update(r.as_raw_slice());
    let hash = hasher.finalize();
    hash.as_bytes().view_bits().to_bitvec()
}
