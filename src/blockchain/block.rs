use std::fmt::Display;

use super::Transaction;

pub struct Block {
    transaction: Transaction
}

impl Block {
    pub fn new(transaction: Transaction) -> Block {
        return Block { transaction };
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}\n{}\n{}\n",
                      "==========================", 
                      self.transaction, 
                      "==========================");
    }
}
