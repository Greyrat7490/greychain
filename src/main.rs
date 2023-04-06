mod wallet;
mod net;
mod blockchain;
mod crypto;

use net::network::Node;
use wallet::Wallet;

use crate::net::network::init_master_nodes;

extern crate rsa;
extern crate rand;
extern crate digest;

fn main() {
    let wallet1 = Wallet::new();

    let master_nodes = vec![Node{ pub_key: wallet1.pub_key_pem.clone(), port: wallet1.port}];
    init_master_nodes(master_nodes);

    let wallet2 = Wallet::new();
    let wallet3 = Wallet::new();

    wallet1.send_tx(&wallet2.pub_key_pem, 420.69);
    wallet1.send_tx(&wallet3.pub_key_pem, 420.69);

    wallet2.send_tx(&wallet1.pub_key_pem, 69.64);
    wallet2.send_tx(&wallet3.pub_key_pem, 69.64);

    wallet3.send_tx(&wallet1.pub_key_pem, 64.420);
    wallet3.send_tx(&wallet2.pub_key_pem, 64.420);

    wallet1.show_network();
    wallet2.show_network();
    wallet3.show_network();

    wallet1.shutdown();
    wallet2.shutdown();
    wallet3.shutdown();
}
