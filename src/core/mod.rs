use std::fmt::Debug;
use std::str::FromStr;

use thiserror::Error;

use crate::crypto;
use crate::crypto::SignatureScheme;

pub mod blocks;
pub mod contract;
pub mod digest;
pub mod hash;
pub mod header;
pub mod number;

pub type Sha3_256 = crate::core::hash::Sha3Hasher;
pub type Header = crate::core::header::Header<Sha3_256>;
pub type Block = crate::core::blocks::Block<Sha3_256>;

pub use contract::{Circuit, CircuitProof, ContractId, ContractPayment, ContractState};

macro_rules! auto_trait {
    (
        $(
            $(#[$doc:meta])+
            trait $name:ident: $( $bound:path ),+;
        )+
    ) => {
        $(
            $(#[$doc])+
            pub trait $name: $( $bound + )+ {}
            impl <T: $($bound +)+> $name for T {}
        )+
    };
}

auto_trait!(
    /// A type that implements Serialize in node runtime
    trait AutoSerialize: serde::ser::Serialize;

    /// A type that implements Deserialize in node runtime
    trait AutoDeserialize: serde::de::DeserializeOwned;
    /// A type that implements Hash in node runtime
    trait AutoHash: core::hash::Hash;
    /// A type that implements Display in runtime
    trait AutoDisplay: core::fmt::Display;
    /// A type that implements CanBe32Bits
    trait CanBe32Bits: core::convert::From<u32>;
);

/// A type that can be used at runtime
pub trait MemberBound: Send + Sync + Sized + Debug + Clone + Eq + PartialEq + 'static {}
impl<T: Send + Sync + Sized + Debug + Clone + Eq + PartialEq + 'static> MemberBound for T {}

pub trait Hash: Debug + Clone + 'static {
    /// The length in bytes of the Hasher output
    const LENGTH: usize;

    type Output: MemberBound
        + AutoSerialize
        + AutoDeserialize
        + AutoHash
        + AsRef<[u8]>
        + AsMut<[u8]>
        + Default
        + Copy
        + PartialOrd;

    fn hash(s: &[u8]) -> Self::Output;
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub enum Signature {
    Unsigned,
    Signed(crypto::Signature),
}

pub type Money = u64;

// All of the Zeeka's supply exists in Treasury account when the blockchain begins.
// Validator/Miner fees are collected from the Treasury account. This simplifies
// the process of money creation.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub enum Address {
    Treasury,
    PublicKey(crypto::PublicKey),
}

#[derive(Error, Debug)]
pub enum ParseAddressError {
    #[error("address invalid")]
    Invalid,
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Address::Treasury => write!(f, "Treasury"),
            Address::PublicKey(pk) => write!(f, "{}", pk),
        }
    }
}

impl FromStr for Address {
    type Err = ParseAddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Address::PublicKey(crypto::PublicKey::from_str(s).unwrap()))
    }
}

// A transaction could be as simple as sending some funds, or as complicated as
// creating a smart-contract.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub enum TransactionData {
    RegularSend {
        dst: Address,
        amount: Money,
    },
    RegisterValidator {
        vrf_stuff: u8,
        amount: Money,
    },

    // Create a Zero-Contract. The creator can consider multiple ways (Circuits) of updating
    // the state. But there should be only one circuit for entering and exiting the contract.
    CreateContract {
        deposit_withdraw_circuit: Circuit,
        update_circuits: Vec<Circuit>,
        initial_state: ContractState,
    },
    // Proof for DepositWithdrawCircuit(curr_state, next_state, hash(entries))
    DepositWithdraw {
        contract_id: ContractId,
        deposit_withdraws: Vec<ContractPayment>,
        next_state: ContractState,
        proof: CircuitProof,
    },
    // Proof for UpdateCircuit[circuit_index](curr_state, next_state)
    Update {
        contract_id: ContractId,
        circuit_index: u32,
        next_state: ContractState,
        proof: CircuitProof,
    },
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Transaction {
    pub src: Address,
    pub nonce: u32,
    pub data: TransactionData,
    pub fee: Money,
    pub sig: Signature,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Account {
    pub balance: Money,
    pub nonce: u32,
}

impl Transaction {
    pub fn hash<H: Hash>(&self) -> H::Output {
        H::hash(&bincode::serialize(self).unwrap())
    }
    pub fn verify_signature(&self) -> bool {
        match &self.src {
            Address::Treasury => true,
            Address::PublicKey(pk) => match &self.sig {
                Signature::Unsigned => false,
                Signature::Signed(sig) => {
                    let mut unsigned = self.clone();
                    unsigned.sig = Signature::Unsigned;
                    let bytes = bincode::serialize(&unsigned).unwrap();
                    crypto::EdDSA::verify(&pk, &bytes, &sig)
                }
            },
        }
    }
}

impl Eq for Transaction {}
impl std::hash::Hash for Transaction {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write(&bincode::serialize(self).unwrap());
        state.finish();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(1, 1)
    }
}
