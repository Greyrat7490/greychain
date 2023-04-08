use std::{
    thread::{JoinHandle, spawn},
    sync::{mpsc::{Receiver, Sender, channel}, Arc, Mutex},
    hash::{Hash, Hasher},
    collections::{hash_map::DefaultHasher, VecDeque}
};

use rand::random;

use super::{Transaction, Block};

const DIFFICULTY: u64 = u64::MAX >> 24;

pub struct Miner {
    queue: VecDeque<Block>,
    send_req: Sender<u64>,
    recv_res: Receiver<u64>,
    online: Arc<Mutex<bool>>,
    thread: JoinHandle<()>,
}

impl Miner {
    pub fn new() -> Miner {
        let (send_req, recv_req) = channel::<u64>();
        let (send_res, recv_res) = channel::<u64>();
        let queue = VecDeque::<Block>::new();

        let online = Arc::new(Mutex::new(true));

        let thread = Self::create_thread(Arc::clone(&online), recv_req, send_res);
        return Miner { queue, send_req, recv_res, online, thread }
    }

    pub fn add_tx(&mut self, tx: Transaction, prev_hash: u64, round: usize) {
        // TODO no blocks with same round
        let (block, nonce) = Block::new_invalid(tx, prev_hash, round);

        self.queue.push_back(block);
        self.send_req.send(nonce);
    }

    pub fn recv_block(&mut self) -> Option<Block> {
        if let Ok(solution) = self.recv_res.try_recv() {
            if let Some(mut block) = self.queue.pop_front() {
                block.complete(solution);
                println!("mined block for tx id {}", block.get_tx_id());
                return Some(block);
            }
        }

        return None;
    }

    pub fn shutdown(self) {
        *self.online.lock().unwrap() = false;
        self.thread.join().unwrap();
    }

    pub fn gen_mining_hash(nonce: u64, solution: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        nonce.hash(&mut hasher);
        solution.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn is_idling(&self) -> bool {
        return self.queue.is_empty();
    }

    fn create_thread(online: Arc<Mutex<bool>>, recv: Receiver<u64>, send: Sender<u64>) -> JoinHandle<()> {
        return spawn(move || {
            loop {
                if let Ok(nonce) = recv.try_recv() {
                    let solution = Self::mine(nonce);
                    send.send(solution);
                } else if !*online.lock().unwrap() {
                    break;
                }
            }
        });
    }

    fn mine(nonce: u64) -> u64 {
        println!("mining...");
        let mut solution = random::<u64>();

        while !Self::verify(nonce, solution) {
            solution = random::<u64>();
        }

        println!("found solution: {}", Self::gen_mining_hash(nonce, solution));
        return solution;
    }

    fn verify(nonce: u64, solution: u64) -> bool {
        return Self::gen_mining_hash(nonce, solution) < DIFFICULTY;
    }
}
