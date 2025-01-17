use crate::core::{Address, Money, Signature, Transaction, TransactionData};
use crate::crypto::{EdDSA, SignatureScheme};

#[derive(Debug, Clone)]
pub struct Wallet {
    seed: Vec<u8>,
    nonce: u32,
}

impl Wallet {
    pub fn new(seed: Vec<u8>) -> Self {
        Self { seed, nonce: 0 }
    }
    pub fn get_address(&self) -> Address {
        let (pk, _) = EdDSA::generate_keys(&self.seed);
        Address::PublicKey(pk)
    }
    pub fn create_transaction(&self, dst: Address, amount: Money, fee: Money) -> Transaction {
        let (_, sk) = EdDSA::generate_keys(&self.seed);
        let mut tx = Transaction {
            src: self.get_address(),
            data: TransactionData::RegularSend { dst, amount },
            nonce: self.nonce,
            fee,
            sig: Signature::Unsigned,
        };
        let bytes = bincode::serialize(&tx).unwrap();
        tx.sig = Signature::Signed(EdDSA::sign(&sk, &bytes));
        tx
    }
}
