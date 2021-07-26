use crate::bits::from_bits;
use crate::core::{PlotSeed, PoSpace};
use crate::Bits;
use bitvec::view::BitView;
use log::*;

type QualityString = Vec<u8>;

#[derive(Debug)]
pub struct Proof {
    pub x_values: Vec<u64>,
    pub challenge: Vec<u8>,
    pub k: usize,
    pub plot_seed: PlotSeed,
}

pub struct Prover {
    pospace: PoSpace,
}

impl Prover {
    pub fn new(pospace: PoSpace) -> Self {
        Prover { pospace }
    }

    /// Require only 6 disk seeks
    pub fn get_quality_string(&self, challenge: &[u8]) -> QualityString {
        let chall_id = from_bits(&challenge.view_bits());
        let qual_id = chall_id % 32;
        let target: Bits = challenge.view_bits()[0..self.pospace.k].to_bitvec();
        todo!()
    }

    pub fn retrieve_all_proofs(&self, challenge: &[u8]) -> Vec<Proof> {
        let target: Bits = challenge.view_bits()[0..self.pospace.k].to_bitvec();
        let proofs = self.pospace.find_xvalues_from_target(&target);
        let proofs = proofs
            .iter()
            .map(|p| {
                let x_values = p
                    .iter()
                    .map(|e| from_bits(&e.metadata.as_ref().unwrap().view_bits()[..self.pospace.k]))
                    .collect::<Vec<u64>>();
                return Proof {
                    x_values,
                    challenge: challenge.to_owned(),
                    k: self.pospace.k,
                    plot_seed: self.pospace.plot_seed,
                };
            })
            .collect::<Vec<Proof>>();
        debug!("Proofs: {:?}", proofs);
        return proofs;
    }
}

#[cfg(test)]
mod tests {}
