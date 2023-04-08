use std::{fmt::Display, hash::{Hash, Hasher}, collections::hash_map::DefaultHasher, time::{SystemTime, UNIX_EPOCH}};

use crate::net::serialize::Serializer;

use super::Transaction;

pub struct Block {
    pub prev_hash: u64,
    pub round: usize,
    pub timestamp: u128,
    transaction: Transaction,
    pub hash: u64,
}

impl Block {
    pub fn new(transaction: Transaction, prev_hash: u64, round: usize) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        let hash = Self::gen_hash(&transaction, prev_hash, round, timestamp);
        return Block { prev_hash, transaction, hash, round, timestamp };
    }

    fn gen_hash(transaction: &Transaction, prev_hash: u64, round: usize, timestamp: u128) -> u64 {
        let mut hasher = DefaultHasher::new();
        prev_hash.hash(&mut hasher);
        round.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        transaction.hash(&mut hasher);

        return hasher.finish();
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}\nhash: {} (prev)\nround: {}\ntimestamp: {}\n{}hash: {} (cur)\n{}\n",
                      "==========================",
                      self.prev_hash,
                      self.round,
                      self.timestamp,
                      self.transaction,
                      self.hash,
                      "==========================");
    }
}

impl Serializer for Block {
    fn serialize(&self, dst: &mut [u8]) -> usize {
        let mut start: usize = 0;

        start += self.prev_hash.serialize(&mut dst[start..]);
        start += self.round.serialize(&mut dst[start..]);
        start += self.timestamp.serialize(&mut dst[start..]);
        start += self.transaction.serialize(&mut dst[start..]);
        start += self.hash.serialize(&mut dst[start..]);

        return start;
    }

    fn deserialize(bytes: &[u8]) -> (usize, Self) {
        let mut start: usize = 0;

        let (size, prev_hash) = u64::deserialize(&bytes[start..]);
        start += size;

        let (size, round) = usize::deserialize(&bytes[start..]);
        start += size;

        let (size, timestamp) = u128::deserialize(&bytes[start..]);
        start += size;

        let (size, transaction) = Transaction::deserialize(&bytes[start..]);
        start += size;

        let (size, hash) = u64::deserialize(&bytes[start..]);
        start += size;

        return (start, Block{ prev_hash, round, timestamp, transaction, hash });
    }
}
