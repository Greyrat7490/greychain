use rsa::{RsaPrivateKey, RsaPublicKey};

pub const RSA_BITS: usize = 2048;
pub const RSA_PEM_SIZE: usize = 52 + RSA_BITS/4/64 + RSA_BITS/4;
pub const RSA_BYTES: usize = RSA_BITS/8;

pub fn create_key_pair() -> (RsaPublicKey, RsaPrivateKey) {
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
    let pub_key = RsaPublicKey::from(&priv_key);

    return (pub_key, priv_key);
}

#[cfg(test)]
mod tests {
    use rsa::{
        RsaPrivateKey, RsaPublicKey,
        pss::{BlindedSigningKey, VerifyingKey},
        sha2::Sha256,
        signature::{Keypair, RandomizedSigner, Verifier}
    };

    use crate::crypto::RSA_BITS;

    #[test]
    pub fn sign() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = sign_key.verifying_key();

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        veri_key.verify(msg.as_bytes(), &sign).unwrap();
    }

    #[test]
    pub fn sign_verify_key_from_pub() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = VerifyingKey::<Sha256>::from(pub_key);

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        veri_key.verify(msg.as_bytes(), &sign).unwrap();
    }

    #[test]
    pub fn sign_verify_key_from_pub_fail() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = VerifyingKey::<Sha256>::from(pub_key);

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        let wrong_msg = "wrong test message";
        if let Ok(()) = veri_key.verify(wrong_msg.as_bytes(), &sign) {
            println!("verify should return error");
            assert!(false);
        }
    }
}
