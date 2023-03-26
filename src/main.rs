mod wallet;
use wallet::Wallet;

fn main() {
    println!("Hello, world!");

    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();

    println!("wallet1 port: {}", wallet1.port);
    println!("wallet2 port: {}", wallet2.port);
}
