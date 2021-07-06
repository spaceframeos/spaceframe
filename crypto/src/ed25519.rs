use crate::traits::{PrivateKey, PublicKey};

struct Ed25519PrivateKey {}

impl PrivateKey for Ed25519PrivateKey {}

struct Ed25519PublicKey {}

impl PublicKey for Ed25519PublicKey {}
