use bitvec::prelude::*;

use crate::constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_M};
use crate::core::collation_size_bits;
use crate::storage::PlotEntry;
use crate::{constants::PARAM_EXT, Bits, BitsSlice};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct RmapItem {
    count: u64,
    pos: u64,
}

#[derive(Debug)]
pub struct FxCalculator {
    k: usize,
    table_index: usize,
    f_size: usize,
    left_targets: Vec<Vec<Vec<u64>>>,
    rmap: Vec<RmapItem>,
    rmap_clean: Vec<u64>,
}

impl FxCalculator {
    pub fn new(k: usize, table_index: usize) -> Self {
        let mut fx = FxCalculator {
            k,
            table_index,
            f_size: k + PARAM_EXT,
            left_targets: vec![vec![vec![0; PARAM_M as usize]; PARAM_BC as usize]; 2],
            rmap: vec![RmapItem { count: 0, pos: 0 }; PARAM_BC as usize],
            rmap_clean: vec![],
        };
        fx.load_tables();
        fx
    }

    pub fn calculate_fn(
        &self,
        y1: &BitsSlice,
        left: &BitsSlice,
        right: &BitsSlice,
    ) -> (Bits, Bits) {
        let mut input = Bits::new();
        let mut c = Bits::new();
        let mut hasher = blake3::Hasher::new();

        if self.table_index < 4 {
            c.extend_from_bitslice(left);
            c.extend_from_bitslice(right);
        }

        input.extend_from_bitslice(y1);
        input.extend_from_bitslice(left);
        input.extend_from_bitslice(right);

        hasher.update(input.as_raw_slice());

        let hash = hasher.finalize().as_bytes().view_bits::<Lsb0>().to_bitvec();
        let output = hash[0..(self.k + PARAM_EXT)].to_bitvec();

        if self.table_index >= 4 && self.table_index < 7 {
            c = hash[0..collation_size_bits(self.table_index + 1, self.k)].to_bitvec();
        }

        return (output, c);
    }

    fn load_tables(&mut self) {
        for parity in 0..2 {
            for i in 0..PARAM_BC {
                let ind_j = i / PARAM_C;
                for m in 0..PARAM_M {
                    let yr = ((ind_j + m) % PARAM_B) * PARAM_C
                        + (((2 * m + parity) * (2 * m + parity) + i) % PARAM_C);
                    self.left_targets[parity as usize][i as usize][m as usize] = yr;
                }
            }
        }
    }

    pub fn find_matches(
        &mut self,
        left_bucket: &[PlotEntry],
        right_bucket: &[PlotEntry],
    ) -> Vec<Match> {
        let mut matches = Vec::new();
        let parity = (left_bucket[0].fx / PARAM_BC) % 2;

        for yl in &self.rmap_clean {
            self.rmap[*yl as usize].count = 0;
        }
        self.rmap_clean.clear();

        let remove = (right_bucket[0].fx / PARAM_BC) * PARAM_BC;
        for pos_r in 0..right_bucket.len() {
            let r_y = (right_bucket[pos_r].fx - remove) as usize;

            if self.rmap[r_y].count == 0 {
                self.rmap[r_y].pos = pos_r as u64;
            }

            self.rmap[r_y].count += 1;
            self.rmap_clean.push(r_y as u64);
        }

        let remove_y = remove - PARAM_BC;
        for pos_l in 0..left_bucket.len() {
            let r = left_bucket[pos_l].fx - remove_y;
            for i in 0..PARAM_M {
                let r_target = self.left_targets[parity as usize][r as usize][i as usize];
                for j in 0..self.rmap[r_target as usize].count {
                    matches.push(Match {
                        left_index: pos_l,
                        right_index: (self.rmap[r_target as usize].pos + j) as usize,
                    });
                }
            }
        }
        return matches;
    }
}

#[derive(Debug)]
pub struct Match {
    pub left_index: usize,
    pub right_index: usize,
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::bits::{from_bits, to_bits, BitsWrapper};
    use crate::f1_calculator::F1Calculator;
    use std::collections::BTreeMap;

    fn matching_naive(l: u64, r: u64) -> bool {
        let k_bc = PARAM_BC as i128;
        let k_b = PARAM_B as i128;
        let k_c = PARAM_C as i128;

        let yl = l as i128;
        let yr = r as i128;

        let bl = yl / k_bc;
        let br = yr / k_bc;

        if bl + 1 != br {
            return false;
        }

        for m in 0i128..PARAM_M as i128 {
            if (((yr % k_bc) / k_c - ((yl % k_bc) / k_c)) - m) % k_b == 0 {
                let mut c_diff = 2 * m + (bl % 2);
                c_diff *= c_diff;

                if (((yr % k_bc) % k_c - ((yl % k_bc) % k_c)) - c_diff) % k_c == 0 {
                    return true;
                }
            }
        }
        return false;
    }

    fn verify_fc(t: usize, k: usize, left: u64, right: u64, y1: u64, y: u64, c: Option<u64>) {
        let sizes = [1, 2, 4, 4, 3, 2];
        let size = sizes[(t - 2) as usize];
        let fcalc = FxCalculator::new(k, t);

        let res = fcalc.calculate_fn(
            &to_bits(y1, k + PARAM_EXT),
            &to_bits(left, k * size),
            &to_bits(right, k * size),
        );

        assert_eq!(y, from_bits(&res.0));
        assert_eq!(k + PARAM_EXT, res.0.len());

        if c.is_some() {
            assert_eq!(c.unwrap(), from_bits(&res.1));
        }
    }

    #[test]
    fn test_matching() {
        const TEST_K: usize = 12;
        const NUM_BUCKETS: u64 = (1u64 << (TEST_K + PARAM_EXT)) / PARAM_BC + 1;
        let test_key: [u8; 32] = [
            20, 2, 5, 4, 51, 52, 23, 84, 91, 10, 111, 12, 13, 24, 151, 16, 228, 211, 254, 45, 92,
            198, 204, 10, 9, 10, 11, 129, 139, 171, 15, 18,
        ];
        let f1 = F1Calculator::new(TEST_K, &test_key);
        let mut x: u64 = 0;
        let mut buckets = BTreeMap::new();

        for _ in 0..((1u64 << (TEST_K - 4)) + 1) {
            let mut y = [0; 1 << 4];

            for i in 0..(1 << 4) {
                y[i as usize] = from_bits(
                    &f1.calculate_f1(&BitsWrapper::from(x * (1 << 4) + i as u64, TEST_K)),
                );
            }

            for i in 0..(1 << 4) {
                let bucket = y[i] / PARAM_BC;
                if !buckets.contains_key(&bucket) {
                    buckets.insert(bucket, vec![]);
                }
                buckets
                    .get_mut(&bucket)
                    .unwrap()
                    .push((to_bits(y[i], TEST_K + PARAM_EXT), to_bits(x, TEST_K)));
                if x + 1 > (1u64 << TEST_K) - 1 {
                    break;
                }
                x += 1;
            }
            if x + 1 > (1u64 << TEST_K) - 1 {
                break;
            }
        }

        let mut f2 = FxCalculator::new(TEST_K, 2);
        let mut total_matches = 0;

        for kv in &buckets {
            if *kv.0 == NUM_BUCKETS - 1 {
                continue;
            }
            let next_bucket = buckets.get(&(kv.0 + 1)).expect("No following bucket");
            let mut left_bucket = Vec::new();
            let mut right_bucket = Vec::new();

            for yx1 in kv.1 {
                let e = PlotEntry {
                    fx: from_bits(&yx1.0),
                    metadata: None,
                    position: None,
                    offset: None,
                };
                left_bucket.push(e);
            }
            for yx2 in next_bucket {
                let e = PlotEntry {
                    fx: from_bits(&yx2.0),
                    metadata: None,
                    position: None,
                    offset: None,
                };
                right_bucket.push(e);
            }

            left_bucket.sort_unstable();
            right_bucket.sort_unstable();

            println!(
                "Bucket {}: {} --> {}",
                kv.0,
                left_bucket[0].fx,
                left_bucket[left_bucket.len() - 1].fx
            );
            println!(
                "Left size: {}, right size: {}",
                left_bucket.len(),
                right_bucket.len(),
            );

            let matches = f2.find_matches(&left_bucket, &right_bucket);
            total_matches += matches.len();
            for m in matches {
                assert!(matching_naive(
                    left_bucket[m.left_index].fx,
                    right_bucket[m.right_index].fx,
                ));
            }
        }

        println!("Total matches: {}", total_matches);

        const MIN_MATCHES: usize = 1 << TEST_K - 1;
        const MAX_MATCHES: usize = 1 << TEST_K + 1;

        assert!(
            total_matches > MIN_MATCHES,
            "Too few matches: {} matches found, minimum is {}",
            total_matches,
            MIN_MATCHES
        );
        assert!(
            total_matches < MAX_MATCHES,
            "Too many matches: {} matches found, maximum is {}",
            total_matches,
            MAX_MATCHES
        );
    }

    #[test]
    fn test_fx() {
        verify_fc(2, 16, 0x44cb, 0x204f, 0x20a61a, 0x39274C, Some(0x44CB204F));
        verify_fc(2, 16, 0x3c5f, 0xfda9, 0x3988ec, 0x30181B, Some(0x3c5ffda9));
        verify_fc(
            3,
            16,
            0x35bf992d,
            0x7ce42c82,
            0x31e541,
            0x26B38B,
            Some(0x35bf992d7ce42c82),
        );
        // verify_fc(
        //     3,
        //     16,
        //     0x7204e52d,
        //     0xf1fd42a2,
        //     0x28a188,
        //     0x3fb0b5,
        //     Some(0x7204e52df1fd42a2),
        // );
        // verify_fc(
        //     4,
        //     16,
        //     0x5b6e6e307d4bedc,
        //     0x8a9a021ea648a7dd,
        //     0x30cb4c,
        //     0x11ad5,
        //     Some(0xd4bd0b144fc26138),
        // );
        // verify_fc(
        //     4,
        //     16,
        //     0xb9d179e06c0fd4f5,
        //     0xf06d3fef701966a0,
        //     0x1dd5b6,
        //     0xe69a2,
        //     Some(0xd02115f512009d4d),
        // );
        // verify_fc(
        //     5,
        //     16,
        //     0xc2cd789a380208a9,
        //     0x19999e3fa46d6753,
        //     0x25f01e,
        //     0x1f22bd,
        //     Some(0xabe423040a33),
        // );
        // verify_fc(
        //     5,
        //     16,
        //     0xbe3edc0a1ef2a4f0,
        //     0x4da98f1d3099fdf5,
        //     0x3feb18,
        //     0x31501e,
        //     Some(0x7300a3a03ac5),
        // );
        // verify_fc(
        //     6,
        //     16,
        //     0xc965815a47c5,
        //     0xf5e008d6af57,
        //     0x1f121a,
        //     0x1cabbe,
        //     Some(0xc8cc6947),
        // );
        // verify_fc(
        //     6,
        //     16,
        //     0xd420677f6cbd,
        //     0x5894aa2ca1af,
        //     0x2efde9,
        //     0xc2121,
        //     Some(0x421bb8ec),
        // );
        // verify_fc(7, 16, 0x5fec898f, 0x82283d15, 0x14f410, 0x24c3c2, None);
        // verify_fc(7, 16, 0x64ac5db9, 0x7923986, 0x590fd, 0x1c74a2, None);
    }
}
