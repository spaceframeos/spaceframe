use crate::traits::{PrivateKey, PublicKey, Signature};
use ed25519_dalek::PublicKey as DalekPublicKey;
use ed25519_dalek::{Digest, Keypair, Sha512, SignatureError};
use ed25519_dalek::{ExpandedSecretKey, SecretKey as DalekPrivateKey, Signature as DalekSignature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ed25519KeyPair {
    pub public: Ed25519PublicKey,
    pub private: Ed25519PrivateKey,
}

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let keypair = Keypair::generate(&mut OsRng);
        Ed25519KeyPair {
            public: Ed25519PublicKey(keypair.public),
            private: Ed25519PrivateKey(keypair.secret),
        }
    }

    pub fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<Ed25519Signature, SignatureError>
    where
        T: AsRef<[u8]>,
    {
        self.private.sign(message, context, self.public)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ed25519PrivateKey(DalekPrivateKey);

impl PrivateKey for Ed25519PrivateKey {
    type SignatureType = Ed25519Signature;
    type PublicKeyType = Ed25519PublicKey;

    fn public_key(&self) -> Self::PublicKeyType {
        Ed25519PublicKey(DalekPublicKey::from(&self.0))
    }

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
        public_key: Self::PublicKeyType,
    ) -> Result<Self::SignatureType, SignatureError>
    where
        T: AsRef<[u8]>,
    {
        let mut hasher = Sha512::new();
        hasher.update(message.as_ref());

        let expanded: ExpandedSecretKey = (&self.0).into();
        let signature: DalekSignature = expanded.sign_prehashed(hasher, &public_key.0, context)?;
        return Ok(Ed25519Signature(signature));
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Ed25519PublicKey(DalekPublicKey);

impl PublicKey for Ed25519PublicKey {
    type SignatureType = Ed25519Signature;

    fn verify<T: AsRef<[u8]>>(
        &self,
        signature: &Self::SignatureType,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<(), SignatureError> {
        let mut hasher = Sha512::new();
        hasher.update(message.as_ref());
        self.0.verify_prehashed(hasher, context, &signature.0)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Ed25519Signature(DalekSignature);

impl Ed25519Signature {
    pub fn zero() -> Self {
        Ed25519Signature(DalekSignature::new([0u8; 64]))
    }
}

impl Signature for Ed25519Signature {}
