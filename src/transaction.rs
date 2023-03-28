use std::{fmt::Display, mem::{transmute, size_of, size_of_val}, sync::atomic::AtomicU64};

use rsa::{
    RsaPublicKey,
    RsaPrivateKey,
    pkcs8::{EncodePublicKey, DecodePublicKey},
    sha2::Sha256,
    pss::{BlindedSigningKey, Signature, VerifyingKey},
    signature::{RandomizedSigner, Verifier}
};

use crate::package::{PKG_SIZE, PKG_CONTENT_SIZE};

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

        let content = b"tmp content"; // TODO: id, amount, payer and payee to bytes as content
        let sign = sign_key.sign_with_rng(&mut rng, content);

        let payer = payer.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let payee = payee.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();

        return Transaction { id: get_next_id(), payer: payer.to_owned(), payee: payee.to_owned(), amount, sign };
    }

    pub fn from(pkg: [u8; PKG_SIZE]) -> Transaction {
        let mut start: usize = 1;
        let mut end: usize = start + size_of::<u64>();

        let id: u64 = unsafe { 
            let ptr = pkg[start..].as_ptr() as *const u64;
            (*ptr).clone()
        };
        start += size_of::<u64>();
        end += size_of::<f64>();

        let amount: f64 = unsafe { 
            let ptr = pkg[start..].as_ptr() as *const f64;
            (*ptr).clone()
        };
        start += size_of::<f64>();
        end += size_of::<usize>();

        let payer_size: usize = unsafe { 
            let ptr = pkg[start..].as_ptr() as *const usize;
            (*ptr).clone()
        };
        start += size_of::<usize>();
        end += payer_size;
        let payer: String = String::from_utf8_lossy(&pkg[start..end]).to_string();
        start += payer_size;
        end += size_of::<usize>();

        let payee_size: usize = unsafe { 
            let ptr = pkg[start..].as_ptr() as *const usize;
            (*ptr).clone()
        };
        start += size_of::<usize>();
        end += payee_size;
        let payee: String = String::from_utf8_lossy(&pkg[start..end]).to_string();
        start += payee_size;
        end +=  size_of::<usize>();

        let sign_size: usize = unsafe { 
            let ptr = pkg[start..].as_ptr() as *const usize;
            (*ptr).clone()
        };
        start += size_of::<usize>();
        end += sign_size;
        let sign = Signature::try_from(&pkg[start..end]).unwrap();

        return Transaction { id, amount, payer, payee, sign };
    }

    pub fn as_bytes(&self) -> [u8; PKG_CONTENT_SIZE] {
        let mut buf: [u8; PKG_CONTENT_SIZE] = [0; PKG_CONTENT_SIZE];
        let mut start: usize = 0;
        let mut end: usize = size_of::<u64>();

        buf[start..end].copy_from_slice(unsafe { &transmute::<u64, [u8; 8]>(self.id) });
        start += size_of::<u64>();
        end += size_of::<f64>();
        buf[start..end].copy_from_slice(unsafe { &transmute::<f64, [u8; 8]>(self.amount) });
        start += size_of::<f64>();
        end += size_of::<usize>();

        buf[start..end].copy_from_slice(unsafe { &transmute::<usize, [u8; 8]>(self.payer.len()) });
        start += size_of::<usize>();
        end += self.payer.len();
        buf[start..end].copy_from_slice(self.payer.as_bytes());
        start += self.payer.len();
        end += size_of::<usize>();

        buf[start..end].copy_from_slice(unsafe { &transmute::<usize, [u8; 8]>(self.payee.len()) });
        start += size_of::<usize>();
        end += self.payee.len();
        buf[start..end].copy_from_slice(self.payee.as_bytes());
        start += self.payee.len();
        end += size_of::<usize>();

        let sign_as_bytes = Box::<[u8]>::from(self.sign.clone());
        let sign_size = size_of_val(&*sign_as_bytes);
        buf[start..end].copy_from_slice(unsafe { &transmute::<usize, [u8; 8]>(sign_size) });
        start += size_of::<usize>();
        end += sign_size;
        buf[start..end].copy_from_slice(&*sign_as_bytes);

        return buf;
    }

    pub fn verify(self) -> bool {
        let pub_key = RsaPublicKey::from_public_key_pem(&self.payer)
            .expect("ERROR: could not get public key from pem");
        let verify_key = VerifyingKey::<Sha256>::from(pub_key);
        let content = b"tmp content"; // TODO: id, amount, payer and payee to bytes as content
        if let Ok(..) = verify_key.verify(content, &self.sign) {
            return true;
        } else {
            println!("ERROR: invalid transaction (corrupted)");
            return false;
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "id: {}\namount: {} GRY\npayer:\n{}payee:\n{}", self.id, self.amount, self.payer, self.payee);
    }
}
