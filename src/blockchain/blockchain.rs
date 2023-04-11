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

    pub fn add_block(&mut self, block: &Block) {
        if self.blocks.is_empty() {
            self.blocks.push(block.to_owned());
        } else {
            let round = block.round;

            if self.verify(&block, round) {
                // better block found
                if block < &self.blocks[round] {
                    println!("better block (round: {})", round);

                    if self.blocks[round].tx == block.tx {
                        self.blocks[round] = block.to_owned();
                        println!("discard tx id {}", block.tx.id);
                    } else {
                        self.blocks.retain(|b| b.tx != block.tx);
                        self.blocks.insert(round, block.to_owned());
                    }

                    self.rehash(round);
                } else {
                    for (i, b) in self.blocks[round..].iter().enumerate() {
                        if b.tx == block.tx {
                            // discard block
                            println!("discard tx id {}", block.tx.id);
                            return;
                        }

                        if b.get_minig_hash() > block.get_minig_hash() {
                            self.blocks.retain(|b| b.tx != block.tx);
                            self.blocks.insert(i, block.to_owned());
                            self.rehash(i);
                            return;
                        }
                    }
                    
                    self.blocks.push(block.to_owned());
                }
            } else {
                println!("hashes are not matching");
                println!("hashes: {} -> {}", self.blocks[round-1].hash, block.prev_hash);
                println!("round: {}", round);
            }
        }   
    }

    pub fn get_round(&self) -> usize {
        return self.blocks.len();
    }

    pub fn get_prev_hash(&self, round: usize) -> u64 {
        if round < 1 || round > self.blocks.len() {
            return 0x0;
        }

        return self.blocks[round-1].hash;
    }

    pub fn get_tx_ids(&self) -> Vec<u64> {
        return self.blocks.iter().map(|b| b.tx.id).collect();
    }

    fn verify(&self, block: &Block, round: usize) -> bool {
        if round == 0 {
            return block.prev_hash == 0x0;
        }

        return block.prev_hash == self.blocks[round-1].hash;
    }

    fn rehash(&mut self, round: usize) {
        let mut prev_hash = self.blocks[round].hash;
        for b in &mut self.blocks[round..] {
            b.rehash(prev_hash);
            prev_hash = b.hash;
        }

        self.cur_hash = prev_hash;
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
