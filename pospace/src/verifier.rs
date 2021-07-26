use crate::bits::{from_bits, to_bits, BitsWrapper};
use crate::f1_calculator::F1Calculator;
use crate::fx_calculator::FxCalculator;
use crate::proofs::Proof;
use crate::storage::PlotEntry;
use bitvec::order::Lsb0;
use bitvec::view::BitView;

pub struct Verifier {}

impl Verifier {
    pub fn new() -> Self {
        Verifier {}
    }

    pub fn get_quality_string(&self) {
        todo!()
    }

    pub fn verify_proof(&self, proof: &Proof) -> bool {
        let f1_calculator = F1Calculator::new(proof.k, proof.plot_seed);

        let mut fx_values = Vec::new();
        let mut metadata = Vec::new();

        for x in &proof.x_values {
            let fx = f1_calculator
                .calculate_f1(&BitsWrapper::from(*x, proof.k))
                .unwrap();
            fx_values.push(fx);
            metadata.push(to_bits(*x, proof.k));
        }

        assert_eq!(64, fx_values.len());

        for table_index in 2..8 {
            let mut fx_calculator = FxCalculator::new(proof.k, table_index);
            let mut temp_fx_values = Vec::new();
            let mut temp_metadata = Vec::new();

            for i in (0..(1 << (8 - table_index))).step_by(2) {
                let left_entry = PlotEntry {
                    fx: from_bits(&fx_values[i]),
                    metadata: None,
                    position: None,
                    offset: None,
                };
                let right_entry = PlotEntry {
                    fx: from_bits(&fx_values[i + 1]),
                    metadata: None,
                    position: None,
                    offset: None,
                };
                let left_bucket = vec![left_entry];
                let right_bucket = vec![right_entry];

                if fx_calculator
                    .find_matches(&left_bucket, &right_bucket)
                    .len()
                    != 1
                {
                    return false;
                }

                let res = fx_calculator.calculate_fn(&fx_values[i], &metadata[i], &metadata[i + 1]);
                temp_fx_values.push(res.0);
                temp_metadata.push(res.1);
            }
            fx_values = temp_fx_values;
            metadata = temp_metadata;
        }

        return fx_values[0][0..proof.k] == proof.challenge.view_bits::<Lsb0>()[0..proof.k];
    }
}
