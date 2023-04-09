mod wallet;
mod net;
mod blockchain;
mod crypto;

use std::{time::Duration, thread::sleep};

use wallet::Wallet;

use crate::net::{tcp::get_pkgs_send, node::Node};

extern crate rsa;
extern crate rand;
extern crate digest;

fn main() {
    let wallet1 = Wallet::new(&vec![]);

    let master_nodes = vec![Node{ pub_key: wallet1.pub_key_pem.clone(), port: wallet1.port, online: true}];

    let wallet2 = Wallet::new(&master_nodes);
    let wallet3 = Wallet::new(&master_nodes);

    wallet1.send_tx(&wallet2.pub_key_pem, 420.69);
    wallet1.send_tx(&wallet3.pub_key_pem, 420.69);

    wallet2.send_tx(&wallet1.pub_key_pem, 69.64);
    wallet2.send_tx(&wallet3.pub_key_pem, 69.64);
    
    wallet3.send_tx(&wallet1.pub_key_pem, 64.420);
    wallet3.send_tx(&wallet2.pub_key_pem, 64.420);

    wallet1.show_network();
    wallet2.show_network();
    wallet3.show_network();

    while !wallet1.is_idling() || !wallet2.is_idling() || !wallet3.is_idling() { 
        sleep(Duration::from_millis(100));
    }     

    let txs = wallet1.get_tx_ids();

    wallet1.shutdown();
    wallet2.shutdown();
    wallet3.shutdown();

    println!("pkgs total send: {}", get_pkgs_send()); 
    println!("txs: {:?}", txs); 
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, thread::sleep};

    use crate::{wallet::Wallet, net::node::Node};

    #[test]
    fn network_3wallets() {
        complete_network(3);
    }

    #[test]
    fn network_6wallets() {
        complete_network(6);
    }

    #[test]
    fn network_6wallets_wait() {
        complete_network_wait_secs(6, 2);
    }

    #[test]
    fn network_15wallets() {
        complete_network(15);
    }

    #[test]
    fn blockchain_equal_3wallets_2tx() {
        blockchain_equal(3, 2);
    }

    #[test]
    fn no_missing_txs_3wallet_2tx() {
        no_missing_txs(3, 2);
    }

    #[test]
    fn no_missing_txs_4wallet_4tx() {
        no_missing_txs(4, 4);
    }



    fn complete_network(wallets_count: usize) {
        let wallets = create_test_wallets(wallets_count);

        wait_for_wallets(&wallets);

        for wallet in &wallets {
            wallet.show_network(); // to see networks when failed
            assert_eq!(wallet.get_network_len(), wallets_count-1);
        }

        shutdown_test_wallets(wallets);
    }

    fn complete_network_wait_secs(wallets_count: usize, secs: u64) {
        let wallets = create_test_wallets(wallets_count);

        sleep(Duration::from_secs(secs));

        for wallet in &wallets {
            wallet.show_network();
            assert_eq!(wallet.get_network_len(), wallets_count-1);
        }

        shutdown_test_wallets(wallets);
    }

    fn blockchain_equal(wallets_count: usize, txs_count: usize) {
        let wallets = create_test_wallets(wallets_count);

        create_txs(&wallets, txs_count);

        wait_for_wallets(&wallets);

        let cur_hash = wallets[0].get_cur_hash();
        for wallet in &wallets[1..] {
            assert_eq!(wallet.get_cur_hash(), cur_hash);
        }

        shutdown_test_wallets(wallets);
    }

    fn no_missing_txs(wallets_count: usize, txs_count: usize) {
        let wallets = create_test_wallets(wallets_count);

        create_txs(&wallets, txs_count);

        wait_for_wallets(&wallets);

        for wallet in &wallets {
            assert_eq!(wallet.get_tx_ids().len(), txs_count);
        }

        shutdown_test_wallets(wallets);
    }

        
    fn create_test_wallets(wallets_count: usize) -> Vec<Wallet> {
        let mut wallets = Vec::<Wallet>::with_capacity(wallets_count);
        wallets.push(Wallet::new(&vec![]));

        let master_nodes = vec![Node{ pub_key: wallets[0].pub_key_pem.clone(), port: wallets[0].port, online: true}];
        wallets.resize_with(wallets_count, || { Wallet::new(&master_nodes) });

        return wallets;
    }

    fn create_txs(wallets: &Vec<Wallet>, txs_count: usize) {
        for i in 0..wallets.len() {
            for mut j in 0..txs_count {
                if j == i { j += 1; }
                let idx = j % wallets.len();

                wallets[i].send_tx(&wallets[idx].pub_key_pem, rand::random::<f64>() * 100.0);
            }
        }
    }

    fn shutdown_test_wallets(wallets: Vec<Wallet>) {
        for wallet in wallets {
            wallet.shutdown();
        }
    }

    fn wait_for_wallets(wallets: &Vec<Wallet>) {
        // wait until all wallets are idling
        loop {
            let mut idling = false;
            for wallet in wallets {
                idling = wallet.is_idling();
                if !idling { break; }
            }

            if idling { break; }

            sleep(Duration::from_millis(100));
        }
    }
}
