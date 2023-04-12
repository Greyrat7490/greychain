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
    const WALLETS_COUNT: usize = 7;
    const TXS_PER_WALLET: usize = 3;

    let wallets = create_test_wallets(WALLETS_COUNT);

    create_txs(&wallets, TXS_PER_WALLET);

    wait_for_wallets(&wallets);

    wallets[0].show_network();

    let txs = wallets[0].get_tx_ids();
    let hashes = wallets.iter().map(|w| w.get_cur_hash()).collect::<Vec<u64>>();
    let blockchain_hashes = wallets[0].get_blockchain_hashes();
    let net_lens = wallets.iter().map(|w| w.get_network_len()).collect::<Vec<usize>>();

    shutdown_test_wallets(wallets);

    println!("\n-------------------"); 
    println!("pkgs total send: {}", get_pkgs_send()); 
    println!("net_lens: {:?}", net_lens);
    println!("hashes:\n{}", hashes.iter().map(|h| {h.to_string() + "\n"}).collect::<String>());
    println!("blockchain hashes:\n{}", blockchain_hashes.iter()
             .map(|(prev, cur)| prev.to_string() + "\n" + &cur.to_string() + "\n")
             .collect::<String>());
    println!("txs: {:?}", txs); 
    println!("txs count: {:?}", txs.len()); 
    println!("-------------------"); 
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, thread::sleep};

    use crate::{wait_for_wallets, create_test_wallets, shutdown_test_wallets, create_txs};

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
    fn txs_3wallets_2tx() {
        check_txs(3, 2);
    }

    #[test]
    fn txs_4wallets_4tx() {
        check_txs(4, 4);
    }

    #[test]
    fn txs_14wallets_4tx() {
        check_txs(14, 4);
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

    fn check_txs(wallets_count: usize, pre_wallet_txs_count: usize) {
        let wallets = create_test_wallets(wallets_count);

        create_txs(&wallets, pre_wallet_txs_count);

        wait_for_wallets(&wallets);

        let txs_total = (pre_wallet_txs_count*wallets_count).try_into().expect("cound not cast u64 into usize");
        let expected = (0..txs_total).collect::<Vec<u64>>();
        for wallet in &wallets {
            let mut res = wallet.get_tx_ids();
            res.sort_unstable();

            // tx ids will not always start from 0 (global id counter)
            let offset = *res.first().expect("expected at least one tx");
            res = res.iter_mut().map(|id| *id - offset).collect();

            assert_eq!(res, expected);
        }

        shutdown_test_wallets(wallets);
    }       
}

fn create_test_wallets(wallets_count: usize) -> Vec<Wallet> {
    let mut wallets = Vec::<Wallet>::with_capacity(wallets_count);
    wallets.push(Wallet::new_master_node());

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
