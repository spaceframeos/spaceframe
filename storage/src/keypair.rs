use std::{
    fs::{read_dir, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::Result;
use spaceframe_crypto::{
    ed25519::{Ed25519KeyPair, Ed25519PrivateKey},
    traits::PrivateKey,
};
use spaceframe_ledger::account::Address;

pub fn store_keypair(keypair: &Ed25519KeyPair, path: &Path) -> Result<()> {
    let address = Address::from(keypair.public);
    let mut file = File::create(path.join(address.to_string()))?;
    let bytes = keypair.private.as_bytes();
    file.write_all(bytes)?;
    Ok(())
}

/// Used for demo command
pub fn read_all_keypair(path: &Path) -> Result<Vec<Ed25519KeyPair>> {
    read_dir(path)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|e| e.is_file())
        .map(|e| {
            let mut file = File::open(e)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let private_key = Ed25519PrivateKey::from_bytes(&buf)?;
            let pub_key = private_key.public_key();
            Ok(Ed25519KeyPair {
                public: pub_key,
                private: private_key,
            })
        })
        .collect::<Result<Vec<Ed25519KeyPair>>>()
}
