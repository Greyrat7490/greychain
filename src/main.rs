mod wallet;
use wallet::Wallet;

fn main() {
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();

    println!("wallet1 port: {}", wallet1.port);
    println!("wallet2 port: {}", wallet2.port);

    wallet1.send(wallet2.port, "send message from wallet1 to wallet2");
    wallet2.send(wallet1.port, "send message from wallet2 to wallet1");

    wallet1.shutdown();
    wallet2.shutdown();
}
