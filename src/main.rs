mod wallet;
mod net;
mod blockchain;
mod crypto;

use net::network::Node;
use rsa::pkcs8::EncodePublicKey;
use wallet::Wallet;

use crate::net::network::init_master_nodes;

extern crate rsa;
extern crate rand;
extern crate digest;

fn main() {
    let wallet1 = Wallet::new();

    let pub_key_pem = wallet1.pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
    let master_nodes = vec![Node{ pub_key: pub_key_pem, port: wallet1.port}];
    init_master_nodes(master_nodes);

    let wallet2 = Wallet::new();
    let wallet3 = Wallet::new();

    wallet1.send_tx(wallet2.port, &wallet2.pub_key, 420.69);
    wallet1.send_tx(wallet3.port, &wallet3.pub_key, 420.69);

    wallet2.send_tx(wallet1.port, &wallet1.pub_key, 69.64);
    wallet2.send_tx(wallet3.port, &wallet3.pub_key, 69.64);

    wallet3.send_tx(wallet1.port, &wallet1.pub_key, 64.420);
    wallet3.send_tx(wallet2.port, &wallet2.pub_key, 64.420);

    wallet1.show_network();
    wallet2.show_network();
    wallet3.show_network();

    wallet1.shutdown();
    wallet2.shutdown();
    wallet3.shutdown();
}
