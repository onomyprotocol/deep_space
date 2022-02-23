#![warn(clippy::all)]
#![allow(clippy::pedantic)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod address;
pub mod client;
pub mod coin;
pub mod decimal;
pub mod error;
pub mod mnemonic;
pub mod msg;
pub mod private_key;
pub mod public_key;
pub mod signature;
pub mod utils;

pub use address::Address;
pub use client::Contact;
pub use coin::Coin;
pub use coin::Fee;
pub use mnemonic::Mnemonic;
pub use msg::Msg;
pub use private_key::MessageArgs;
pub use private_key::PrivateKey;
pub use public_key::PublicKey;
pub use signature::Signature;

pub use u64_array_bigints::u256;
pub use u64_array_bigints::U256 as Uint256;
