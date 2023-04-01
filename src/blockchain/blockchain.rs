use std::fmt::Display;

use super::Block;

pub struct Blockchain {
    blocks: Vec<Block>
}

impl Blockchain {
    pub fn new() -> Blockchain {
        return Blockchain{ blocks: Vec::new() };
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
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
