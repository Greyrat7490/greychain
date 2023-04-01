use std::{fmt::Display, mem::size_of, sync::atomic::AtomicU64};

use rsa::{
    RsaPublicKey,
    RsaPrivateKey,
    pkcs8::{EncodePublicKey, DecodePublicKey},
    sha2::Sha256,
    pss::{BlindedSigningKey, Signature, VerifyingKey},
    signature::{RandomizedSigner, Verifier}
};

use crate::net::{pkg::{PKG_SIZE, PKG_CONTENT_SIZE}, serialize::Serializer};

#[derive(Clone)]
pub struct Transaction {
    id: u64,
    amount: f64,
    payer: String,
    payee: String,
    sign: Signature
}

fn get_next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    return NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Release);
}

impl Transaction {
    pub fn new(payer: &RsaPublicKey, payee: &RsaPublicKey,
               priv_key: &RsaPrivateKey, amount: f64) -> Transaction {
        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key.clone());
        let mut rng = rand::thread_rng();

        let payer = payer.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let payee = payee.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();

        let id = get_next_id();

        let mut content = vec![0u8; PKG_CONTENT_SIZE];
        serialize_content(id, amount, &payer, &payee, &mut content);

        let sign = sign_key.sign_with_rng(&mut rng, &content);

        return Transaction { id, payer: payer.to_owned(), payee: payee.to_owned(), amount, sign };
    }

    pub fn deserialize(pkg: [u8; PKG_CONTENT_SIZE]) -> Transaction {
        let mut start: usize = 0;

        let id = u64::deserialize(&pkg[start..]);
        start += size_of::<u64>();

        let amount = f64::deserialize(&pkg[start..]);
        start += size_of::<f64>();

        let payer = String::deserialize(&pkg[start..]);
        start += payer.len() + size_of::<usize>();

        let payee = String::deserialize(&pkg[start..]);
        start += payee.len() + size_of::<usize>();

        let sign = Signature::deserialize(&pkg[start..]);

        return Transaction { id, amount, payer, payee, sign };
    }

    pub fn serialize(&self) -> [u8; PKG_CONTENT_SIZE] {
        let mut buf: [u8; PKG_CONTENT_SIZE] = [0; PKG_CONTENT_SIZE];

        let start = serialize_content(self.id, self.amount, &self.payer, &self.payee, &mut buf);

        self.sign.serialize(&mut buf[start..]);

        return buf;
    }

    pub fn verify(self) -> bool {
        let pub_key = RsaPublicKey::from_public_key_pem(&self.payer)
            .expect("ERROR: could not get public key from pem");
        let verify_key = VerifyingKey::<Sha256>::from(pub_key);

        let mut content = vec![0u8; PKG_CONTENT_SIZE];
        serialize_content(self.id, self.amount, &self.payer, &self.payee, &mut content);

        if let Ok(..) = verify_key.verify(&content, &self.sign) {
            return true;
        } else {
            println!("ERROR: invalid transaction (corrupted)");
            return false;
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "id: {}\namount: {} GRY\npayer:\n{}payee:\n{}",
                      self.id, self.amount, self.payer, &self.payee[..self.payee.len()-1]);
    }
}


fn serialize_content(id: u64, amount: f64, payer: &String, payee: &String, dst: &mut [u8]) -> usize {
    let mut start: usize = 0;

    id.serialize(&mut dst[start..]);
    start += size_of::<u64>();

    amount.serialize(&mut dst[start..]);
    start += size_of::<f64>();

    payer.serialize(&mut dst[start..]);
    start += payer.len() + size_of::<usize>();

    payee.serialize(&mut dst[start..]);
    start += payee.len() + size_of::<usize>();

    return start
}
