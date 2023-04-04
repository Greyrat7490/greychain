mod wallet;
mod net;
mod blockchain;
mod crypto;

use wallet::Wallet;

extern crate rsa;
extern crate rand;
extern crate digest;

fn main() {
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    let wallet3 = Wallet::new();

    println!("{}: is now on", wallet1.get_name());
    println!("{}: is now on", wallet2.get_name());
    println!("{}: is now on", wallet3.get_name());

    wallet1.send_tx(wallet2.port, &wallet2.pub_key, 420.69);
    wallet1.send_tx(wallet3.port, &wallet3.pub_key, 420.69);

    wallet2.send_tx(wallet1.port, &wallet1.pub_key, 69.64);
    wallet2.send_tx(wallet3.port, &wallet3.pub_key, 69.64);

    wallet3.send_tx(wallet1.port, &wallet1.pub_key, 64.420);
    wallet3.send_tx(wallet2.port, &wallet2.pub_key, 64.420);

    wallet1.shutdown();
    wallet2.shutdown();
    wallet3.shutdown();
}
