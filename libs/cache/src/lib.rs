//! Caching utilities.
#![warn(missing_docs)]

use sha2::{Digest, Sha256};

pub mod client;
pub mod error;
pub mod mem;
pub mod rpc;
pub mod server;

pub(crate) fn hash(val: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(val);
    hasher.finalize()[..].into()
}
