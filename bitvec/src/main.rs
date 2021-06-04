use bitvec::prelude::*;

fn main() {
    let x = 0xabcd as u64;
    let x_bytes_be = x.to_be_bytes();
    let x_bytes_le = x.to_le_bytes();
    println!("Big endian Bytes: {:?}", x_bytes_be);
    println!("Little endian Bytes: {:?}", x_bytes_le);
    let x_bits_be_lsb = &x_bytes_be.view_bits::<Lsb0>()[..10usize];
    let x_bits_be_msb = &x_bytes_be.view_bits::<Msb0>()[..10usize];
    let mut x_bits_le_lsb = x_bytes_le.view_bits::<Lsb0>()[..10usize].to_bitvec();
    let x_bits_le_msb = &x_bytes_le.view_bits::<Msb0>()[..10usize];
    x_bits_le_lsb.reverse();
    println!("Bits BE LSB {}", x_bits_be_lsb);
    println!("Bits BE MSB {}", x_bits_be_msb);
    println!("Bits LE LSB {}", x_bits_le_lsb);
    println!("Bits LE MSB {}", x_bits_le_msb);

    println!("{}", (12 as u8).view_bits::<Msb0>());
    println!("{}", (12 as u8));
    println!("{}", (12 as u8 >> 2).view_bits::<Msb0>());
    println!("{}", (12 as u8 >> 2));
}
