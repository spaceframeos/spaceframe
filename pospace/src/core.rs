use bitvec::prelude::*;

use crate::{f1_calculator::F1Calculator, fx_calculator::calculate_f2, utils::matching};

#[derive(Debug)]
pub struct PoSpace {
    plot_seed: String,
    k: usize,
    f1_calculator: F1Calculator
}

impl PoSpace {

    pub fn new(k: usize, plot_seed: &str) -> Self {
        PoSpace {
            plot_seed: plot_seed.to_owned(),
            k,
            f1_calculator: F1Calculator::new(k, plot_seed.as_bytes())
        }
    }

    pub fn run_phase_1(&self) {
        let mut table1 = vec![];
        let mut table2 = vec![];
        for x in 0..(2u64).pow(self.k as u32) {
            let fx = self.f1_calculator.calculate_f1(&x.to_be_bytes().view_bits()[(64 - self.k) as usize..]);
            table1.push(fx);
        }
        println!("Table 1 len: {}", table1.len());
        let mut counter = 0;

        // Table 2
        'outer: for x1 in 0..(2u64).pow(self.k as u32) {
            for x2 in x1..(2u64).pow(self.k as u32) {
                if x1 != x2 {
                    let fx1 = &table1[x1 as usize];
                    let fx2 = &table1[x2 as usize];
                    if matching(fx1, fx2) {
                        let f2x = calculate_f2(
                            &x1.to_be_bytes().view_bits()[(64 - self.k) as usize..],
                            &x2.to_be_bytes().view_bits()[(64 - self.k) as usize..],
                            fx1,
                        );
                        // println!("f2x = {}, x1 = {}, x2 = {}", f2x, x1, x2);
                        counter += 1;
                        table2.push((f2x, x1, x2));

                        if counter == (2u64).pow(self.k as u32) {
                            break 'outer;
                        }
                    }
                }
            }
        }
        println!("Count: {}", counter);
        println!("Table 2 len: {}", table2.len());
    }

}