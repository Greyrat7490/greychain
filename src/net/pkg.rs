use std::{mem::{self, size_of}, fmt::Display};

use crate::transaction::Transaction;

pub const PKG_SIZE: usize = size_of::<Package>();
pub const PKG_CONTENT_SIZE: usize = 2000;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PackageType {
    Msg, Tx, Block, Fork
}

#[repr(C, packed)]
#[derive(Clone)]
pub struct Package {
    pub typ: PackageType,
    pub content: [u8; PKG_CONTENT_SIZE]
}

impl Package {
    pub fn new_msg(s: &str) -> Package {
        let mut content: [u8; PKG_CONTENT_SIZE] = [0; PKG_CONTENT_SIZE];
        content[..s.len()].copy_from_slice(&s.as_bytes());
        return Package{ typ: PackageType::Msg, content };
    }

    pub fn new_tx(tx: Transaction) -> Package {
        return Package{ typ: PackageType::Tx, content: tx.serialize() };
    }

    pub fn deserialize(bytes: [u8; PKG_SIZE]) -> Package {
        return unsafe { mem::transmute(bytes) }
    }

    pub fn serialize(&self) -> [u8; PKG_SIZE] {
        return unsafe { mem::transmute(self.clone()) }
    }

    pub fn verify(&self) -> bool {
        match self.typ {
            PackageType::Msg => {
                // TODO
                println!("TODO: verify msg Package in work (always false for now)");
                return false;
            }

            PackageType::Tx => {
                return Transaction::deserialize(self.serialize()).verify();
            }

            PackageType::Block => { return false; }
            PackageType::Fork => { return false;  }
        };
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_str = match self.typ {
            PackageType::Msg => {
                String::from_utf8_lossy(&self.content).to_string()
            }

            PackageType::Tx => {
                Transaction::deserialize(self.serialize()).to_string()
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

