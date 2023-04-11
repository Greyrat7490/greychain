use std::fmt::Display;

use super::Block;

pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        return Blockchain{ blocks: Vec::new() };
    }

    pub fn add_block(&mut self, block: &Block) {
        if block.round >= self.blocks.len() {
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
                            let first_change_idx = self.blocks.iter()
                                .position(|b| b.tx != block.tx)
                                .unwrap_or(i);
                            self.blocks.retain(|b| b.tx != block.tx);
                            self.blocks.insert(i, block.to_owned());
                            self.rehash(first_change_idx);
                            return;
                        }
                    }

                    let mut b = block.to_owned();
                    b.rehash(self.get_cur_hash());
                    self.blocks.push(b);
                }
            } else {
                println!("hashes are not matching");
                println!("hashes: {} -> {}", self.get_prev_hash(round), block.prev_hash);
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

    pub fn get_cur_hash(&self) -> u64 {
        if let Some(block) = self.blocks.last() {
            return block.hash;
        }

        return 0x0;
    }

    pub fn get_hashes(&self) -> Vec<(u64, u64)> {
        return self.blocks.iter().map(|b| (b.prev_hash, b.hash)).collect::<Vec<(u64, u64)>>();
    }

    fn verify(&self, block: &Block, round: usize) -> bool {
        if round == 0 {
            return block.prev_hash == 0x0;
        }

        return block.prev_hash == self.blocks[round-1].hash;
    }

    fn rehash(&mut self, round: usize) {
        let mut prev_hash = self.blocks[round].hash;
        for b in &mut self.blocks[round+1..] {
            b.rehash(prev_hash);
            prev_hash = b.hash;
        }
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
