use std::{mem, fmt::Display};

pub const PACKAGE_SIZE: usize = 50;

#[derive(Debug, Clone, Copy)]
pub enum PackageType {
    Msg, Tx
}

#[repr(C, packed)]
pub struct Package {
    pub typ: PackageType,
    pub msg: [u8; PACKAGE_SIZE-1]
}

impl Package {
    pub fn new(typ: PackageType, s: &str) -> Package {
        let mut msg: [u8; PACKAGE_SIZE-1] = [0; PACKAGE_SIZE-1];
        msg[..s.len()].copy_from_slice(&s.as_bytes());
        return Package{ typ, msg };
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
        return write!(f, "TYPE: {:?} {{\n  {}\n}}", self.typ, String::from_utf8_lossy(&self.as_bytes()));
    }
}

