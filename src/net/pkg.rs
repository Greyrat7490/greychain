use std::{mem::{self, size_of}, fmt::Display};

use rsa::{
    pss::{Signature, VerifyingKey, BlindedSigningKey},
    sha2::Sha256, RsaPublicKey,
    pkcs8::DecodePublicKey,
    signature::{Verifier, RandomizedSigner}
};

use crate::blockchain::Transaction;

use super::serialize::Serializer;

pub const PKG_CONTENT_SIZE: usize = 2000;
pub const PKG_SIZE: usize = PKG_CONTENT_SIZE + 1000;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PackageType {
    Tx, Block, Fork
}

pub struct Package {
    pub typ: PackageType,
    pub content: [u8; PKG_CONTENT_SIZE],
    sender: String,
    sign: Signature
}

impl Package {
    pub fn new_tx(tx: Transaction, sign_key: BlindedSigningKey<Sha256>) -> Package {
        let content = tx.serialize();

        let mut rng = rand::thread_rng();
        let sign = sign_key.sign_with_rng(&mut rng, &content);

        return Package{ typ: PackageType::Tx, content, sender: tx.payer, sign };
    }

    pub fn deserialize(bytes: [u8; PKG_SIZE]) -> Self {
        let mut start: usize = 0;

        let typ = PackageType::deserialize(&bytes[start..]);
        start += size_of::<PackageType>();

        let mut content = [0u8; PKG_CONTENT_SIZE];
        content.copy_from_slice(&bytes[start..start+PKG_CONTENT_SIZE]); 
        start += PKG_CONTENT_SIZE;

        let sender = String::deserialize(&bytes[start..]);
        start += sender.len() + size_of::<usize>();

        let sign = Signature::deserialize(&bytes[start..]);

        return Package { typ, content, sender, sign };
    }

    pub fn serialize(&self) -> [u8; PKG_SIZE] {
        let mut buf = [0u8; PKG_SIZE];
        let mut start: usize = 0;

        self.typ.serialize(&mut buf[start..]);
        start += size_of::<PackageType>();

        buf[start..start+PKG_CONTENT_SIZE].copy_from_slice(&self.content);
        start += PKG_CONTENT_SIZE;

        self.sender.serialize(&mut buf[start..]);
        start += self.sender.len() + size_of::<usize>();

        self.sign.serialize(&mut buf[start..]);

        return buf;
    }

    pub fn verify(&self) -> bool {
        let pub_key = RsaPublicKey::from_public_key_pem(&self.sender)
            .expect("ERROR: could not get public key from pem");
        let verify_key = VerifyingKey::<Sha256>::from(pub_key);

        if let Ok(..) = verify_key.verify(&self.content, &self.sign) {
            return true;
        } else {
            println!("ERROR: invalid transaction (corrupted)");
            return false;
        }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_str = match self.typ {
            PackageType::Tx => {
                Transaction::deserialize(self.content).to_string()
            }

            PackageType::Block => {
                "BLOCK PACKAGE CONTENT".to_string()
            }

            PackageType::Fork => {
                "FORK PACKAGE CONTENT".to_string()
            }
        };

        return write!(f, "TYPE: {:?} {{\n{}\n}}\n", self.typ, content_str);
    }
}

