use ed25519_dalek::SignatureError;
use std::fmt::Debug;

pub trait Keypair: Debug {
    type PublicKeyType: PublicKey;
    type PrivateKeyType: PrivateKey;

    fn generate() -> Self;

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<<<Self as Keypair>::PublicKeyType as PublicKey>::SignatureType, SignatureError>
    where
        T: AsRef<[u8]>;

    fn public_key(&self) -> Self::PublicKeyType;

    fn private_key(&self) -> &Self::PrivateKeyType;
}

pub trait PrivateKey: Debug {
    type SignatureType: Signature;
    type PublicKeyType: PublicKey;

    fn public_key(&self) -> Self::PublicKeyType;

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
        public_key: Self::PublicKeyType,
    ) -> Result<Self::SignatureType, SignatureError>
    where
        T: AsRef<[u8]>;

    fn as_bytes(&self) -> &[u8];
}

pub trait PublicKey: Copy + Clone + PartialEq + Debug {
    type SignatureType: Signature;

    fn verify<T: AsRef<[u8]>>(
        &self,
        signature: &Self::SignatureType,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<(), SignatureError>;

    fn as_bytes(&self) -> &[u8];
}

pub trait Signature: Clone + PartialEq + Debug {}
