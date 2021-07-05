use std::fmt::Display;

use hex::encode;

pub type Hash = Vec<u8>;

pub struct MerkleTree {
    tree: Vec<Hash>,
    leaf_count: usize,
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            tree: Vec::new(),
            leaf_count: 0,
        }
    }

    pub fn add(&mut self, data: &[u8]) {
        let hash = Self::hash(data);
        self.tree.insert(self.leaf_count, hash);
        self.leaf_count += 1;
        self.rebuild_tree();
    }

    pub fn root(&self) -> Option<&[u8]> {
        self.tree.last().map(|x| x.as_slice())
    }

    pub fn verify(&self, _data: &[u8]) -> bool {
        todo!()
    }

    fn rebuild_tree(&mut self) {
        let mut leaves = self.tree[..self.leaf_count].to_vec();
        self.tree = leaves.clone();

        while leaves.len() > 1 {
            let parents = leaves
                .chunks(2)
                .map(|data| {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&data[0]);
                    hasher.update(&data.get(1).unwrap_or(&data[0]));
                    hasher.finalize().as_bytes().to_vec()
                })
                .collect::<Vec<Hash>>();
            self.tree.extend(parents.clone());
            leaves = parents;
        }

        println!("Tree rebuilt: \n{}", self);
    }

    fn hash(data: &[u8]) -> Hash {
        blake3::hash(data).as_bytes().to_vec()
    }
}

impl Display for MerkleTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut counter = self.leaf_count - 1;
        let mut leaf_count = self.leaf_count;
        self.tree.iter().for_each(|node| {
            let mut display = encode(node);
            display.truncate(8);

            write!(f, "{}, ", display).unwrap();

            if counter == 0 {
                write!(f, "\n").unwrap();
                leaf_count = (leaf_count as f64 / 2f64).ceil() as usize;
                counter = leaf_count - 1;
            } else {
                counter -= 1;
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_constructor() {
        let merkle = MerkleTree::new();
        assert_eq!(0, merkle.leaf_count);
        assert_eq!(0, merkle.tree.len());
    }

    #[test]
    fn test_merkle_add_one_element() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data");

        assert_eq!(1, merkle.leaf_count);
        assert_eq!(1, merkle.tree.len());
        assert_eq!(MerkleTree::hash(b"data").to_vec(), merkle.tree[0]);
    }

    #[test]
    fn test_merkle_add_two_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");

        assert_eq!(2, merkle.leaf_count);
        assert_eq!(3, merkle.tree.len());
        assert_eq!(MerkleTree::hash(b"data1").to_vec(), merkle.tree[0]);
        assert_eq!(MerkleTree::hash(b"data2").to_vec(), merkle.tree[1]);
    }

    #[test]
    fn test_merkle_add_three_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");
        merkle.add(b"data3");

        assert_eq!(3, merkle.leaf_count);
        assert_eq!(6, merkle.tree.len());
        assert_eq!(MerkleTree::hash(b"data1").to_vec(), merkle.tree[0]);
        assert_eq!(MerkleTree::hash(b"data2").to_vec(), merkle.tree[1]);
        assert_eq!(MerkleTree::hash(b"data3").to_vec(), merkle.tree[2]);
    }

    #[test]
    fn test_merkle_add_four_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");
        merkle.add(b"data3");
        merkle.add(b"data4");

        assert_eq!(4, merkle.leaf_count);
        assert_eq!(7, merkle.tree.len());
        assert_eq!(MerkleTree::hash(b"data1").to_vec(), merkle.tree[0]);
        assert_eq!(MerkleTree::hash(b"data2").to_vec(), merkle.tree[1]);
        assert_eq!(MerkleTree::hash(b"data3").to_vec(), merkle.tree[2]);
        assert_eq!(MerkleTree::hash(b"data4").to_vec(), merkle.tree[3]);
    }

    #[test]
    fn test_merkle_add_five_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");
        merkle.add(b"data3");
        merkle.add(b"data4");
        merkle.add(b"data5");

        assert_eq!(5, merkle.leaf_count);
        assert_eq!(11, merkle.tree.len());
        assert_eq!(MerkleTree::hash(b"data1").to_vec(), merkle.tree[0]);
        assert_eq!(MerkleTree::hash(b"data2").to_vec(), merkle.tree[1]);
        assert_eq!(MerkleTree::hash(b"data3").to_vec(), merkle.tree[2]);
        assert_eq!(MerkleTree::hash(b"data4").to_vec(), merkle.tree[3]);
        assert_eq!(MerkleTree::hash(b"data5").to_vec(), merkle.tree[4]);
    }

    #[test]
    fn test_merkle_root() {
        let mut merkle_tree = MerkleTree::new();

        merkle_tree.add(b"data1");
        merkle_tree.add(b"data2");
        merkle_tree.add(b"data3");
        merkle_tree.add(b"data4");

        assert_eq!(
            hex::decode("a6b764089d73a35323f5bf570e3bc8a803c78953cafe9ff4297233b2c9bc24ba")
                .unwrap(),
            merkle_tree.root().unwrap()
        );
    }

    #[test]
    fn test_check_hash() {
        let mut merkle_tree = MerkleTree::new();

        merkle_tree.add(b"data1");
        merkle_tree.add(b"data2");
        merkle_tree.add(b"data3");
        merkle_tree.add(b"data4");

        assert!(merkle_tree.verify(b"data1"));
    }
}
