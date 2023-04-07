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

    sleep(Duration::from_millis(80));

    wallet1.show_network();
    wallet2.show_network();
    wallet3.show_network();

    wallet1.shutdown();
    wallet2.shutdown();
    wallet3.shutdown();

    println!("pkgs total send: {}", get_pkgs_send()); 
    // 17  -> 15 (with 3 wallets),  9 (tx and shutdown)
    // 34  -> 26 (with 4 wallets), 10 (tx and shutdown)
    // 64  -> 44 (with 5 wallets), 11 (tx and shutdown)
    // 108 -> 71 (with 6 wallets), 12 (tx and shutdown)
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, thread::sleep};

    use crate::{wallet::Wallet, net::node::Node};

    #[test]
    fn network_3wallets() {
        network_with_n_wallets(3);
    }

    #[test]
    fn network_6wallets() {
        network_with_n_wallets(6);
    }

    #[test]
    fn network_15wallets() {
        network_with_n_wallets(15);
    }

    fn network_with_n_wallets(wallets_count: usize) {
        let wallet1 = Wallet::new(&vec![]);

        let master_nodes = vec![Node{ pub_key: wallet1.pub_key_pem.clone(), port: wallet1.port, online: true}];

        let mut wallets = Vec::<Wallet>::with_capacity(wallets_count-1);
        wallets.resize_with(wallets_count-1, || { Wallet::new(&master_nodes) });

        sleep(Duration::from_millis((wallets_count*20).try_into().unwrap()));

        let wallet1_net_len = wallet1.get_network_len();
        let mut network_lens = Vec::<usize>::with_capacity(wallets_count-1);
        for wallet in &wallets {
            network_lens.push(wallet.get_network_len())
        }

        wallet1.show_network();
        for wallet in &wallets {
            wallet.show_network();
        }

        wallet1.shutdown();
        for wallet in wallets {
            wallet.shutdown();
        }

        assert_eq!(wallet1_net_len, wallets_count-1);
        for l in network_lens {
            assert_eq!(l, wallets_count-1);
        }
    }
}
