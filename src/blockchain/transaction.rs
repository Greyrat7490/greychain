use std::{fmt::Display, mem::size_of, sync::atomic::AtomicU64, hash::Hash};

use rsa::{
    RsaPublicKey,
    pkcs8::EncodePublicKey
};

use crate::net::{pkg::PKG_CONTENT_SIZE, serialize::Serializer};

#[derive(Clone)]
pub struct Transaction {
    id: u64,
    amount: f64,
    pub payer: String,
    payee: String,
}

fn get_next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    return NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Release);
}

impl Transaction {
    pub fn new(payer: &RsaPublicKey, payee: &RsaPublicKey, amount: f64) -> Transaction {
        let id = get_next_id();
        let payer = payer.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let payee = payee.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();

        return Transaction { id, payer: payer.to_owned(), payee: payee.to_owned(), amount };
    }

    pub fn deserialize(pkg: [u8; PKG_CONTENT_SIZE]) -> Transaction {
        let mut start: usize = 0;

        let id = u64::deserialize(&pkg[start..]);
        start += size_of::<u64>();

        let amount = f64::deserialize(&pkg[start..]);
        start += size_of::<f64>();

        let payer = String::deserialize(&pkg[start..]);
        start += payer.len() + size_of::<usize>();

        let payee = String::deserialize(&pkg[start..]);

        return Transaction { id, amount, payer, payee };
    }

    pub fn serialize(&self) -> [u8; PKG_CONTENT_SIZE] {
        let mut buf: [u8; PKG_CONTENT_SIZE] = [0; PKG_CONTENT_SIZE];
        let mut start: usize = 0;

        self.id.serialize(&mut buf[start..]);
        start += size_of::<u64>();

        self.amount.serialize(&mut buf[start..]);
        start += size_of::<f64>();

        self.payer.serialize(&mut buf[start..]);
        start += self.payer.len() + size_of::<usize>();

        self.payee.serialize(&mut buf[start..]);

        return buf;
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "id: {}\namount: {} GRY\npayer:\n{}payee:\n{}",
                      self.id, self.amount, self.payer, &self.payee[..self.payee.len()-1]);
    }
}

impl Hash for Transaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.amount.to_be_bytes().hash(state);
        self.payer.hash(state);
        self.payee.hash(state);
    }
}
