mod wallet;
mod package;
mod transaction;

use wallet::Wallet;
use transaction::Transaction;

fn main() {
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();

    println!("wallet1 port: {}", wallet1.port);
    println!("wallet2 port: {}", wallet2.port);

    wallet1.send_msg(wallet2.port, "send message from wallet1 to wallet2");
    wallet2.send_msg(wallet1.port, "send message from wallet2 to wallet1");

    wallet1.send_tx(wallet2.port, Transaction::new("wallet1", "wallet2", 420.69));
    wallet2.send_tx(wallet1.port, Transaction::new("wallet2", "wallet1", 69.64));

    wallet1.shutdown();
    wallet2.shutdown();
}
