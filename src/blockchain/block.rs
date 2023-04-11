use std::{fmt::Display, hash::{Hash, Hasher}, collections::hash_map::DefaultHasher, time::{SystemTime, UNIX_EPOCH}};

use crate::net::serialize::Serializer;

use super::{Transaction, Miner};

#[derive(Clone)]
pub struct Block {
    pub prev_hash: u64,
    pub round: usize,
    pub timestamp: u128,
    pub tx: Transaction,
    nonce: u64,
    solution: u64,
    pub hash: u64,
}

impl Block {
    pub fn new(tx: Transaction, prev_hash: u64, round: usize, nonce: u64, solution: u64) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        let hash = Self::gen_hash(&tx, prev_hash, round, timestamp, nonce, solution);
        return Block { prev_hash, tx, hash, round, timestamp, nonce, solution };
    }

    pub fn rehash(&mut self, new_prev_hash: u64) {
        self.prev_hash = new_prev_hash;
        self.hash = Self::gen_hash(&self.tx, self.prev_hash, self.round, self.timestamp, self.nonce, self.solution);
    }

    pub fn get_minig_hash(&self) -> u64 {
        return Miner::gen_mining_hash(self.nonce, self.solution);
    }

    fn gen_hash(tx: &Transaction, prev_hash: u64, round: usize, timestamp: u128, nonce: u64, solution: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        prev_hash.hash(&mut hasher);
        round.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        tx.hash(&mut hasher);
        nonce.hash(&mut hasher);
        solution.hash(&mut hasher);

        return hasher.finish();
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}\nhash: {} (prev)\nround: {}\ntimestamp: {}\n{}nonce: {}\nsolution: {}\nhash: {} (cur)\n{}\n",
                      "==========================",
                      self.prev_hash,
                      self.round,
                      self.timestamp,
                      self.tx,
                      self.nonce,
                      self.solution,
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
        start += self.tx.serialize(&mut dst[start..]);
        start += self.nonce.serialize(&mut dst[start..]);
        start += self.solution.serialize(&mut dst[start..]);
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

        let (size, tx) = Transaction::deserialize(&bytes[start..]);
        start += size;

        let (size, nonce) = u64::deserialize(&bytes[start..]);
        start += size;

        let (size, solution) = u64::deserialize(&bytes[start..]);
        start += size;

        let (size, hash) = u64::deserialize(&bytes[start..]);
        start += size;

        return (start, Block{ prev_hash, round, timestamp, tx, nonce, solution, hash});
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        let a = Miner::gen_mining_hash(self.nonce, self.solution);
        let b = Miner::gen_mining_hash(other.nonce, other.solution);

        return a == b;
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = Miner::gen_mining_hash(self.nonce, self.solution);
        let b = Miner::gen_mining_hash(other.nonce, other.solution);

        return Some(a.cmp(&b));
    }
}
