use std::{mem, fmt::Display};

use crate::transaction::Transaction;

pub const PACKAGE_SIZE: usize = 200;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PackageType {
    Msg, Tx, Block, Fork
}

#[repr(C, packed)]
pub struct Package {
    pub typ: PackageType,
    pub msg: [u8; PACKAGE_SIZE-1]
}

impl Package {
    pub fn new_msg(s: &str) -> Package {
        let mut msg: [u8; PACKAGE_SIZE-1] = [0; PACKAGE_SIZE-1];
        msg[..s.len()].copy_from_slice(&s.as_bytes());
        return Package{ typ: PackageType::Msg, msg };
    }

    pub fn new_tx(tx: Transaction) -> Package {
        return Package{ typ: PackageType::Tx, msg: tx.as_bytes() };
    }

    pub fn from(bytes: [u8; PACKAGE_SIZE]) -> Package {
        return unsafe { mem::transmute(bytes) }
    }

    pub fn as_bytes(&self) -> [u8; PACKAGE_SIZE] {
        let mut res: [u8; PACKAGE_SIZE] = [0; PACKAGE_SIZE];
        res[0] = self.typ as u8;
        res[1..].copy_from_slice(self.msg.as_slice());
        return res;
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_str = match self.typ {
            PackageType::Msg => {
                String::from_utf8_lossy(&self.msg).to_string()
            }

            PackageType::Tx => {
                Transaction::from(self.as_bytes()).to_string()
            }

            PackageType::Block => {
                "BLOCK PACKAGE CONTENT".to_string()
            }

            PackageType::Fork => {
                "FORK PACKAGE CONTENT".to_string()
            }
        };

        return write!(f, "TYPE: {:?} {{\n  {}\n}}", self.typ, content_str);
    }
}

