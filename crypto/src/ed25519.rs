use crate::error::SignatureError;
use crate::traits::{Keypair, PrivateKey, PublicKey, Signature};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::ed25519::signature::Signature as DalekSignatureTrait;
use ed25519_dalek::ed25519::SIGNATURE_LENGTH;
use ed25519_dalek::{Digest, Keypair as DalekKeypair, Sha512};
use ed25519_dalek::{ExpandedSecretKey, SecretKey as DalekPrivateKey, Signature as DalekSignature};
use ed25519_dalek::{PublicKey as DalekPublicKey, PUBLIC_KEY_LENGTH};
use rand::rngs::OsRng;
use std::io::{Error, ErrorKind, Write};

#[derive(Debug)]
pub struct Ed25519KeyPair {
    pub public: Ed25519PublicKey,
    pub private: Ed25519PrivateKey,
}

impl Keypair for Ed25519KeyPair {
    type PublicKeyType = Ed25519PublicKey;
    type PrivateKeyType = Ed25519PrivateKey;

    fn generate() -> Self {
        let keypair = DalekKeypair::generate(&mut OsRng);
        Ed25519KeyPair {
            public: Ed25519PublicKey(keypair.public),
            private: Ed25519PrivateKey(keypair.secret),
        }
    }

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<<Self::PublicKeyType as PublicKey>::SignatureType>
    where
        T: AsRef<[u8]>,
    {
        self.private.sign(message, context, self.public)
    }

    fn public_key(&self) -> Self::PublicKeyType {
        self.public
    }

    fn private_key(&self) -> &Self::PrivateKeyType {
        &self.private
    }
}

#[derive(Debug)]
pub struct Ed25519PrivateKey(DalekPrivateKey);

impl PrivateKey for Ed25519PrivateKey {
    type PublicKeyType = Ed25519PublicKey;

    fn public_key(&self) -> Self::PublicKeyType {
        Ed25519PublicKey(DalekPublicKey::from(&self.0))
    }

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
        public_key: Self::PublicKeyType,
    ) -> Result<<Self::PublicKeyType as PublicKey>::SignatureType>
    where
        T: AsRef<[u8]>,
    {
        let mut hasher = Sha512::new();
        hasher.update(message.as_ref());

        let expanded: ExpandedSecretKey = (&self.0).into();
        let signature: DalekSignature = expanded
            .sign_prehashed(hasher, &public_key.0, context)
            .or(Err(SignatureError::InvalidSignature))?;
        return Ok(Ed25519Signature(signature));
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Ed25519PrivateKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Ed25519PrivateKey(DalekPrivateKey::from_bytes(bytes)?))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ed25519PublicKey(DalekPublicKey);

impl PublicKey for Ed25519PublicKey {
    type SignatureType = Ed25519Signature;

    fn verify<T: AsRef<[u8]>>(
        &self,
        signature: &Self::SignatureType,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<()> {
        let mut hasher = Sha512::new();
        hasher.update(message.as_ref());
        self.0
            .verify_prehashed(hasher, context, &signature.0)
            .or(Err(SignatureError::InvalidSignature.into()))
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl BorshSerialize for Ed25519PublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

impl BorshDeserialize for Ed25519PublicKey {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let bytes = &buf[0..PUBLIC_KEY_LENGTH];
        let key = DalekPublicKey::from_bytes(bytes).or(Err(Error::new(
            ErrorKind::InvalidData,
            "Could not create public key from bytes",
        )))?;
        *buf = &buf[PUBLIC_KEY_LENGTH..];
        Ok(Ed25519PublicKey(key))
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Ed25519Signature(DalekSignature);

impl Ed25519Signature {
    pub fn zero() -> Self {
        Ed25519Signature(DalekSignature::new([0u8; 64]))
    }
}

impl Signature for Ed25519Signature {}

impl BorshSerialize for Ed25519Signature {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

impl BorshDeserialize for Ed25519Signature {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let bytes = &buf[0..SIGNATURE_LENGTH];
        let signature = DalekSignature::from_bytes(bytes).or(Err(Error::new(
            ErrorKind::InvalidData,
            "Could not create signature key from bytes",
        )))?;
        *buf = &buf[SIGNATURE_LENGTH..];
        Ok(Ed25519Signature(signature))
    }
}
