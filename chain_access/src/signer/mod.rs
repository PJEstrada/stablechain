pub mod local_key;
pub mod privy_signer;

pub use local_key::LocalKeySigner;
pub use privy_signer::PrivySigner;

use alloy::primitives::{Address, Bytes};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;

use crate::error::ChainAccessError;

pub enum SignerBackendType {
    LocalKey,
    Privy,
}

impl SignerBackendType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local-key" => Some(Self::LocalKey),
            "privy" => Some(Self::Privy),
            _ => None,
        }
    }
}

/// Abstraction over a signing backend.
/// Implementations sign a `TransactionRequest` and return the RLP-encoded signed bytes.
/// They know nothing about chains, RPC, or broadcasting.
#[async_trait]
pub trait SignerBackend: Send + Sync {
    /// Returns the Ethereum address controlled by this signer.
    async fn address(&self) -> Result<Address, ChainAccessError>;

    /// Signs the transaction and returns the RLP-encoded signed tx bytes.
    async fn sign_transaction(&self, tx: TransactionRequest) -> Result<Bytes, ChainAccessError>;

    /// A short label for display purposes (e.g. `"local-key"`, `"privy"`, `"privy-user"`).
    fn signer_kind(&self) -> &'static str;
}
