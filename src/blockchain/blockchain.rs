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

    pub fn get_round(&self) -> usize {
        return self.blocks.len();
    }

    fn verify(&self, block: &Block) -> bool {
        return block.prev_hash == self.cur_hash;
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
