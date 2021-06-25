use crate::bits::BitsWrapper;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Table1Entry {
    pub x: u64,
    pub y: u64,
}

pub struct PlotEntry {
    pub y: BitsWrapper,
    pub pos: u64,
    pub offset: u64,
}
