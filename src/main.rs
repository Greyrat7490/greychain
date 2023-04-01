mod wallet;
mod net;
mod blockchain;

use wallet::Wallet;

extern crate rsa;
extern crate rand;
extern crate digest;

fn main() {
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();

    println!("wallet1 port: {}", wallet1.port);
    println!("wallet2 port: {}", wallet2.port);

    wallet1.send_msg(wallet2.port, "send message from wallet1 to wallet2");
    wallet2.send_msg(wallet1.port, "send message from wallet2 to wallet1");

    wallet1.send_tx(wallet2.port, &wallet2.pub_key, 420.69);
    wallet2.send_tx(wallet1.port, &wallet1.pub_key, 69.64);

    wallet1.shutdown();
    wallet2.shutdown();
}
