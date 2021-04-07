use std::ops::{Index, Range};

#[derive(PartialEq, Eq, Debug)]
pub struct SmallVector {
    data: [u64; 10],
    count: usize,
}

impl SmallVector {
    pub fn new() -> Self {
        SmallVector {
            data: [0; 10],
            count: 0
        }
    }

    pub fn size(&self) -> usize {
        self.count
    }

    pub fn push_back(&mut self, value: u64) {
        self.data[self.count] = value;
    }
}

impl Index<Range<usize>> for SmallVector {
    type Output = [u64];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl Index<usize> for SmallVector {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}