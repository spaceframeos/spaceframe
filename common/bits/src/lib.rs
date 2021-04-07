mod smallvector;

use std::{fmt::{Display}, ops::{Index, Range}};

use smallvector::SmallVector;

pub struct BitsGeneric<T> 
    where T: Index<Range<usize>> + Index<usize> {
    data: T,
    last_size: u8,
}

pub type Bits = BitsGeneric<SmallVector>;

impl Bits {
    pub fn empty() -> Self {
        Bits {
            data: SmallVector::new(),
            last_size: 0,
        }
    }

    pub fn new(value: u128, size: u32) -> Self {
        let bits = Self::empty();

        if size > 64 {
            bits.init_bits((value >> 64) as u64, size - 64);
            bits.append(value, size as u8);
        } else {
            bits.init_bits(value as u64, size - 64);
        }

        bits
    }

    fn init_bits(&mut self, value: u64, size: u32) {
        self.last_size = 0;

        if size > 64 {
            let zeros = size - bits_count(value) as u32;
            while zeros > 64 {
                self.append(0, 64);
                zeros -= 64;
            }

            self.append(0, zeros as u8);
            self.append(value as u128, bits_count(value));
        } else {
            assert!(size == 64 || value == (value & ((1 << size) - 1)), "'value' must be under 'size' bits");
            self.data.push_back(value);
            self.last_size = size as u8;
        }
    }

    pub fn append(&mut self, value: u128, length: u8) {
        if length > 64 {
            self.do_append((value >> 64) as u64, length - 64);
            self.do_append(value as u64, 64);
        } else {
            self.do_append(value as u64, length);
        }
    }

    fn do_append(&mut self, value: u64, length: u8) {
        // The last bucket is full or no bucket yet, create a new one.
        if self.data.size() == 0 || self.last_size == 64 {
            self.data.push_back(value);
            self.last_size = length;
        } else {
            let free_bits: u8 = 64 - self.last_size;
            if self.last_size == 0 && length == 64 {
                // Special case for OSX -O3, as per -fsanitize=undefined
                // runtime error: shift exponent 64 is too large for 64-bit type 'uint64_t' (aka
                // 'unsigned long long')
                self.data[self.data.size() - 1] = value;
                self.last_size = length;
            } else if length <= free_bits {
                // If the value fits into the last bucket, append it all there.
                self.data[self.data.size() - 1] = (self.data[self.data.size() - 1] << length) + value;
                self.last_size += length;
            } else {
                // Otherwise, append the prefix into the last bucket, and create a new bucket for
                // the suffix.
                let (prefix, suffix) = split_number_by_prefix(value, length, free_bits);
                self.data[self.data.size() - 1] = (self.data[self.data.size() - 1] << free_bits) + prefix;
                self.data.push_back(suffix);
                self.last_size = length - free_bits;
            }
        }
    }

    pub fn slice(start_index: u32, end_index: u32) -> Bits {
        
    }

    pub fn get_value(&self) -> Option<u64> {
        if self.data.size() > 0 {
            Some(self.data[0])
        } else {
            None
        }
    }
}

fn bits_count(value: u64) -> u8 {
    let mut count = 0;
    while value > 0 {
        count += 1;
        value >>= 1;
    }
    return count;
}

fn split_number_by_prefix(number: u64, num_bits: u8, prefix_size: u8) -> (u64, u64) {
    assert!(num_bits >= prefix_size, "number of bits must be greater or equal than the prefix size");

    if prefix_size == 0 {
        return (0, number);
    } else {
        let suffix_size = num_bits - prefix_size;
        let mut mask = 1 << suffix_size;
        mask -= 1;
        return (number >> suffix_size, number & mask);
    }
}

impl Display for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value() {
        let bits = Bits::new(13271, 15);
        assert_eq!(13271, bits.get_value().unwrap())
    }
}
