use std::{fmt::Display, mem::{transmute, size_of}, sync::atomic::AtomicU64};

use crate::package::PACKAGE_SIZE;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Transaction {
    id: u64,
    amount: f64,
    payer: [u8; 64],
    payee: [u8; 64],
}

fn get_next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    return NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Release);
}

impl Transaction {
    pub fn new(payer: &str, payee: &str, amount: f64) -> Transaction {
        let mut payer_buf: [u8; 64] = [0; 64];
        payer_buf[..payer.len()].copy_from_slice(payer.as_bytes());

        let mut payee_buf: [u8; 64] = [0; 64];
        payee_buf[..payee.len()].copy_from_slice(payee.as_bytes());
        return Transaction { id: get_next_id(), payer: payer_buf, payee: payee_buf, amount };
    }

    pub fn from(pkg: [u8; PACKAGE_SIZE]) -> Transaction {
        const TX_BUF_SIZE: usize = size_of::<Transaction>();

        let mut bytes: [u8; TX_BUF_SIZE] = [0; TX_BUF_SIZE];
        bytes.copy_from_slice(&pkg[1..TX_BUF_SIZE+1]);

        return unsafe { transmute(bytes) };
    }

    pub fn as_bytes(&self) -> [u8; PACKAGE_SIZE-1] {
        const TX_BUF_SIZE: usize = size_of::<Transaction>();

        let tx_buf: [u8; TX_BUF_SIZE] = unsafe { transmute(*self) };
        let mut buf: [u8; PACKAGE_SIZE-1] = [0; PACKAGE_SIZE-1];

        buf[..TX_BUF_SIZE].copy_from_slice(tx_buf.as_slice());
        return buf;
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id;
        let amount = self.amount;

        return write!(f, "id: {} {}GRY {} -> {}", id, amount, 
                      String::from_utf8_lossy(self.payer.as_slice()),
                      String::from_utf8_lossy(self.payee.as_slice()));
    }
}
