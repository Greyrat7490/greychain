use std::{mem::size_of, fmt::Display};

use rsa::{
    pss::{Signature, VerifyingKey, BlindedSigningKey},
    sha2::Sha256, RsaPublicKey,
    pkcs8::DecodePublicKey,
    signature::{Verifier, RandomizedSigner}
};

use crate::{blockchain::Transaction, crypto::{RSA_BYTES, RSA_PEM_SIZE}};

use super::{serialize::Serializer, node::Node};

pub const PKG_CONTENT_SIZE: usize = 9000;                   // TODO: smaller
pub const PKG_SIZE: usize = size_of::<PackageType>() +
                            PKG_CONTENT_SIZE +
                            RSA_PEM_SIZE + size_of::<usize>() +
                            RSA_BYTES + size_of::<usize>() +
                            size_of::<bool>();

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PackageType {
    Tx, Status, NodesRes, Block, Fork
}

#[derive(Clone)]
pub struct Package {
    pub typ: PackageType,
    pub content: [u8; PKG_CONTENT_SIZE],
    pub sender: String,
    sign: Signature,
    pub is_forwarded: bool
}

impl Package {
    pub fn new<T: Serializer>(content: T, typ: PackageType, pub_key: String,
                              sign_key: BlindedSigningKey<Sha256>) -> Package {
        let mut content_bytes = [0u8; PKG_CONTENT_SIZE];
        content.serialize(&mut content_bytes);

        let mut rng = rand::thread_rng();
        let sign = sign_key.sign_with_rng(&mut rng, &content_bytes);

        return Package{ typ, content: content_bytes, sender: pub_key, sign, is_forwarded: false };
    }

    pub fn deserialize(bytes: [u8; PKG_SIZE]) -> Self {
        let mut start: usize = 0;

        let (size, typ) = PackageType::deserialize(&bytes[start..]);
        start += size;

        let mut content = [0u8; PKG_CONTENT_SIZE];
        content.copy_from_slice(&bytes[start..start+PKG_CONTENT_SIZE]);
        start += PKG_CONTENT_SIZE;

        let (size, sender) = String::deserialize(&bytes[start..]);
        start += size;

        let (size, sign) = Signature::deserialize(&bytes[start..]);
        start += size;

        let is_forwarded = bool::deserialize(&bytes[start..]).1;

        return Package { typ, content, sender, sign, is_forwarded };
    }

    pub fn serialize(&self) -> [u8; PKG_SIZE] {
        let mut buf = [0u8; PKG_SIZE];
        let mut start: usize = 0;

        start += self.typ.serialize(&mut buf[start..]);
        buf[start..start+PKG_CONTENT_SIZE].copy_from_slice(&self.content);
        start += PKG_CONTENT_SIZE;
        start += self.sender.serialize(&mut buf[start..]);
        start += self.sign.serialize(&mut buf[start..]); 
        self.is_forwarded.serialize(&mut buf[start..]);

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
                Transaction::deserialize(&self.content).1.to_string()
            }

            PackageType::Block => {
                "BLOCK PACKAGE CONTENT\n".to_string()
            }

            PackageType::Fork => {
                "FORK PACKAGE CONTENT\n".to_string()
            }

            PackageType::NodesRes => {
                let nodes = Vec::<Node>::deserialize(&self.content).1;
                nodes.iter().map(|node| node.to_string() + "\n").collect::<String>()
            }

            PackageType::Status => {
                let node = Node::deserialize(&self.content).1;
                if node.online {
                    "Register wallet:\n".to_string() + &node.pub_key
                } else {
                    "Deregister wallet:\n".to_string() + &node.pub_key
                }
            }
        };

        return write!(f, "TYPE: {:?} {{\n{}}}\n", self.typ, content_str);
    }
}

