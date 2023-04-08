use std::{fmt::Display, sync::atomic::AtomicU64, hash::Hash};

use crate::net::serialize::Serializer;

#[derive(Clone)]
pub struct Transaction {
    pub id: u64,
    amount: f64,
    pub payer: String,
    payee: String,
}

fn get_next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    return NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Release);
}

impl Transaction {
    pub fn new(payer: &String, payee: &String, amount: f64) -> Transaction {
        let id = get_next_id();

        return Transaction { id, payer: payer.to_owned(), payee: payee.to_owned(), amount };
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "id: {}\namount: {} GRY\npayer:\n{}payee:\n{}", self.id, self.amount, self.payer, &self.payee);
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

impl Serializer for Transaction {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        let mut start: usize = 0;

        start += self.id.serialize(&mut dst[start..]);
        start += self.amount.serialize(&mut dst[start..]);
        start += self.payer.serialize(&mut dst[start..]); 
        start += self.payee.serialize(&mut dst[start..]);

        return start;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start: usize = 0;

        let (size, id) = u64::deserialize(&bytes[start..]);
        start += size;

        let (size, amount) = f64::deserialize(&bytes[start..]);
        start += size;

        let (size, payer) = String::deserialize(&bytes[start..]);
        start += size;

        let (size, payee) = String::deserialize(&bytes[start..]);
        start += size;

        return (start, Transaction { id, amount, payer, payee });
    }
}
