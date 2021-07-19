use spaceframe_ledger::ledger::Ledger;

use crate::error::StorageError;
use anyhow::Result;
use borsh::BorshSerialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn write_to_disk(ledger: &Ledger, path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(StorageError::PathIsNotDirectory.into());
    }
    for block in &ledger.blockchain {
        let block_bytes = block
            .try_to_vec()
            .or(Err(StorageError::SerializationError))?;
        let file_path = path.join(format!("block_{}", block.height));
        let mut file = File::create(&file_path).or(Err(StorageError::FileCreationFailed))?;
        file.write_all(&block_bytes)
            .or(Err(StorageError::DataWriteFailed))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshDeserialize;
    use spaceframe_crypto::ed25519::Ed25519KeyPair;
    use spaceframe_crypto::traits::Keypair;
    use spaceframe_ledger::account::Address;
    use spaceframe_ledger::ledger::Block;
    use spaceframe_ledger::transaction::Tx;
    use std::io::Read;
    use tempdir::TempDir;

    #[test]
    fn test_write_ledger_to_disk() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();
        let mut ledger = Ledger::new(&[
            Tx::genesis(&Address::from(keypair.public), 13),
            Tx::genesis(&Address::from(keypair_2.public), 15),
        ])
        .unwrap();

        ledger
            .add_block_from_transactions(&[
                Tx::new(&keypair, &Address::from(keypair_2.public), 6, 1).unwrap(),
                Tx::new(&keypair_2, &Address::from(keypair.public), 5, 1).unwrap(),
            ])
            .unwrap();

        let tmpdir = TempDir::new("test_write_ledger_to_disk").unwrap();
        let res = write_to_disk(&ledger, tmpdir.path());
        assert!(res.is_ok());

        for block in &ledger.blockchain {
            let file_path = tmpdir.path().join(format!("block_{}", block.height));
            let mut file = File::open(&file_path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let block_red = Block::try_from_slice(&buffer).unwrap();
            assert_eq!(block, &block_red);
        }
    }
}
