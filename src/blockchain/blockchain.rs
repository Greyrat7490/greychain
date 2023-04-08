use std::fmt::Display;

use super::Block;

pub struct Blockchain {
    blocks: Vec<Block>,
    pub cur_hash: u64
}

impl Blockchain {
    pub fn new() -> Blockchain {
        return Blockchain{ blocks: Vec::new(), cur_hash: 0x0 };
    }

    pub fn add_block(&mut self, block: Block) {
        if self.verify(&block) {
            self.cur_hash = block.hash;
            self.blocks.push(block);
        } else {
            println!("hashes are not matching");
            println!("prev_hash: {}", self.cur_hash);
            println!("round: {}", self.get_round());
            println!("block: {}", block);
        }
    }

    pub fn resolve_fork(&mut self, block: &Block) -> bool {
        let round = block.round;
        assert!(self.blocks.len() > 0);
 
        if self.verify_prev(&block, round) {
            // better block found -> fork
            if block < &self.blocks[round] {
                println!("fork (round: {}) hash: {} -> {}", round, self.blocks[round].hash, block.hash);
                println!("discard tx ids: {}", self.blocks[round..].iter().map(|b| b.get_tx_id().to_string() + " ").collect::<String>());

                // TODO txs of removed blocks have to be broadcasted again 
                // (if not in new block)
                self.blocks.truncate(round);

                self.cur_hash = block.hash;
                self.blocks.push(block.to_owned());
                return true;
            }
        }

        return false;
    }

    pub fn get_round(&self) -> usize {
        return self.blocks.len();
    }

    fn verify(&self, block: &Block) -> bool {
        return block.prev_hash == self.cur_hash;
    }

    fn verify_prev(&self, block: &Block, round: usize) -> bool {
        return block.prev_hash == self.blocks[round].prev_hash;
    }
}

impl Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "------------ Greychain ------------\n{}",
                      self.blocks
                      .iter()
                      .map(|block| block.to_string())
                      .collect::<String>());
    }
}
