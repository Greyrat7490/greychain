use std::{mem::size_of, fmt::Display};

use rsa::{
    pss::{Signature, VerifyingKey, BlindedSigningKey},
    sha2::Sha256, RsaPublicKey,
    pkcs8::DecodePublicKey,
    signature::{Verifier, RandomizedSigner}
};

use crate::{blockchain::{Transaction, Block}, crypto::{RSA_BYTES, RSA_PEM_SIZE}};

use super::{serialize::Serializer, network::Node};

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

const NODE_SIZE: usize = RSA_PEM_SIZE + size_of::<usize>() + size_of::<u16>();
const NODE_LIST_MAX_LEN: usize = (PKG_CONTENT_SIZE - size_of::<u8>()) / NODE_SIZE;

// TODO: as trait
#[derive(Clone)]
pub struct Package {
    pub typ: PackageType,
    pub content: [u8; PKG_CONTENT_SIZE],
    pub sender: String,
    sign: Signature,
    pub is_forwarded: bool
}

impl Package {
    pub fn new_tx(tx: Transaction, sign_key: BlindedSigningKey<Sha256>) -> Package {
        let mut content = [0u8; PKG_CONTENT_SIZE];
        tx.serialize(&mut content);

        let mut rng = rand::thread_rng();
        let sign = sign_key.sign_with_rng(&mut rng, &content);

        return Package{ typ: PackageType::Tx, content, sender: tx.payer, sign, is_forwarded: false };
    }

    pub fn new_status(pub_key: String, port: u16, go_online: bool, sign_key: BlindedSigningKey<Sha256>) -> Package {
        let mut content = [0u8; PKG_CONTENT_SIZE];
        let mut start: usize = 0;

        start += (go_online as u8).serialize(&mut content[start..]);
        start += pub_key.serialize(&mut content[start..]);
        port.serialize(&mut content[start..]);

        let mut rng = rand::thread_rng();
        let sign = sign_key.sign_with_rng(&mut rng, &content);

        return Package { typ: PackageType::Status, content, sender: pub_key, sign, is_forwarded: false }
    }

    pub fn new_nodes(sender: &String, nodes: Vec<Node>, sign_key: &BlindedSigningKey<Sha256>) -> Package {
        assert!(nodes.len() <= NODE_LIST_MAX_LEN);

        let mut content = [0u8; PKG_CONTENT_SIZE];
        let mut start: usize = 0;

        content[start] = nodes.len() as u8;
        start += size_of::<u8>();

        for node in nodes {
            node.pub_key.serialize(&mut content[start..]);
            start += node.pub_key.len() + size_of::<usize>();

            node.port.serialize(&mut content[start..]);
            start += size_of::<u16>();
        }

        let mut rng = rand::thread_rng();
        let sign = sign_key.sign_with_rng(&mut rng, &content);

        return Package { typ: PackageType::NodesRes, content, sender: sender.to_string(), sign, is_forwarded: false }
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

pub fn deserialize_status(bytes: [u8; PKG_CONTENT_SIZE]) -> (bool, Node) {
    let mut start = 0;

    let (size, status) = bool::deserialize(&bytes[start..]);
    start += size;

    let (size, pub_key) = String::deserialize(&bytes[start..]);
    start += size;

    let port = u16::deserialize(&bytes[start..]).1;

    return (status, Node { pub_key, port });
}

pub fn deserialize_nodes(bytes: [u8; PKG_CONTENT_SIZE]) -> Vec<Node> {
    let mut start = 0;
    let mut nodes = Vec::<Node>::new();

    let len = bytes[0];
    start += 1;

    for _ in 0..len {
        let (size, pub_key) = String::deserialize(&bytes[start..]);
        start += size;

        let (size, port) = u16::deserialize(&bytes[start..]);
        start += size;

        nodes.push(Node { pub_key, port })
    }

    return nodes;
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
                let nodes = deserialize_nodes(self.content);
                if nodes.len() == 0 {
                    "empty\n".to_string()
                } else {
                    nodes.iter().map(|node| node.to_string() + "\n").collect::<String>()
                }
            }

            PackageType::Status => {
                let (status, node) = deserialize_status(self.content);
                if status {
                    "Register wallet:\n".to_string() + &node.pub_key
                } else {
                    "Deregister wallet:\n".to_string() + &node.pub_key
                }
            }
        };

        return write!(f, "TYPE: {:?} {{\n{}}}\n", self.typ, content_str);
    }
}

