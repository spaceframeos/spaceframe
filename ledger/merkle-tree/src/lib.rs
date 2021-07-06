use spaceframe_crypto::Hash;
use std::fmt::Display;

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

    pub fn with_transactions(mut self, transactions: &[&[u8]]) -> Self {
        self.tree = transactions.iter().map(Hash::from).collect();
        self.leaf_count = transactions.len();
        self.rebuild_tree();
        self
    }

    pub fn add(&mut self, data: &[u8]) {
        let hash = Hash::from(data);
        self.tree.insert(self.leaf_count, hash);
        self.leaf_count += 1;
        self.rebuild_tree();
    }

    pub fn root(&self) -> Option<&[u8]> {
        self.tree.last().map(|x| x.bytes())
    }

    fn get_leaves(&self) -> &[Hash] {
        self.tree[..self.leaf_count].as_ref()
    }

    fn rebuild_tree(&mut self) {
        let mut leaves = self.get_leaves().to_vec();
        self.tree = leaves.clone();

        while leaves.len() > 1 {
            let parents = leaves.chunks(2).map(Hash::concat).collect::<Vec<Hash>>();
            self.tree.extend(parents.clone());
            leaves = parents;
        }
    }
}

impl Display for MerkleTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut counter = self.leaf_count - 1;
        let mut leaf_count = self.leaf_count;
        self.tree.iter().for_each(|node| {
            let mut display = node.hex();
            display.truncate(6);

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
    fn test_merkle_with_transactions() {
        let merkle = MerkleTree::new().with_transactions(&[b"data1", b"data2", b"data3", b"data4"]);

        assert_eq!(4, merkle.leaf_count);
        assert_eq!(7, merkle.tree.len());
        assert_eq!(Hash::from(b"data1"), merkle.tree[0]);
        assert_eq!(Hash::from(b"data2"), merkle.tree[1]);
        assert_eq!(Hash::from(b"data3"), merkle.tree[2]);
        assert_eq!(Hash::from(b"data4"), merkle.tree[3]);

        assert_eq!(
            hex::decode("a6b764089d73a35323f5bf570e3bc8a803c78953cafe9ff4297233b2c9bc24ba")
                .unwrap(),
            merkle.root().unwrap()
        );
    }

    #[test]
    fn test_merkle_add_one_element() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data");

        assert_eq!(1, merkle.leaf_count);
        assert_eq!(1, merkle.tree.len());
        assert_eq!(Hash::from(b"data"), merkle.tree[0]);
    }

    #[test]
    fn test_merkle_add_two_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");

        assert_eq!(2, merkle.leaf_count);
        assert_eq!(3, merkle.tree.len());
        assert_eq!(Hash::from(b"data1"), merkle.tree[0]);
        assert_eq!(Hash::from(b"data2"), merkle.tree[1]);
    }

    #[test]
    fn test_merkle_add_three_elements() {
        let mut merkle = MerkleTree::new();
        merkle.add(b"data1");
        merkle.add(b"data2");
        merkle.add(b"data3");

        assert_eq!(3, merkle.leaf_count);
        assert_eq!(6, merkle.tree.len());
        assert_eq!(Hash::from(b"data1"), merkle.tree[0]);
        assert_eq!(Hash::from(b"data2"), merkle.tree[1]);
        assert_eq!(Hash::from(b"data3"), merkle.tree[2]);
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
        assert_eq!(Hash::from(b"data1"), merkle.tree[0]);
        assert_eq!(Hash::from(b"data2"), merkle.tree[1]);
        assert_eq!(Hash::from(b"data3"), merkle.tree[2]);
        assert_eq!(Hash::from(b"data4"), merkle.tree[3]);
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
        assert_eq!(Hash::from(b"data1"), merkle.tree[0]);
        assert_eq!(Hash::from(b"data2"), merkle.tree[1]);
        assert_eq!(Hash::from(b"data3"), merkle.tree[2]);
        assert_eq!(Hash::from(b"data4"), merkle.tree[3]);
        assert_eq!(Hash::from(b"data5"), merkle.tree[4]);
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
}
