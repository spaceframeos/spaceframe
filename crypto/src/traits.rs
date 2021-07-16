use ed25519_dalek::SignatureError;

pub trait Keypair {
    type PublicKeyType;
    type PrivateKeyType;
    type SignatureType;

    fn generate() -> Self;

    fn sign<T>(
        &self,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<Self::SignatureType, SignatureError>
    where
        T: AsRef<[u8]>;

    fn public_key(&self) -> Self::PublicKeyType;

    fn private_key(&self) -> &Self::PrivateKeyType;
}

pub trait PrivateKey {
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

pub trait PublicKey {
    type SignatureType: Signature;

    fn verify<T: AsRef<[u8]>>(
        &self,
        signature: &Self::SignatureType,
        message: T,
        context: Option<&[u8]>,
    ) -> Result<(), SignatureError>;

    fn as_bytes(&self) -> &[u8];
}

pub trait Signature {}

pub trait CryptoSuit {
    type Keypair: Keypair;
    type PubKey: PublicKey;
    type PrivKey: PrivateKey;
    type Signature: Signature;
}
