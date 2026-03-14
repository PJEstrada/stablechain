use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;

use crate::adapters::tempo_provider::{connect_tempo_url, TempoProvider};
use crate::domain::chain_id::ChainId;
use crate::domain::erc20;
use crate::error::ChainAccessError;
use crate::ports::ChainReader;

pub struct TempoAdapter {
    provider: TempoProvider,
    chain_id: ChainId,
}

impl TempoAdapter {
    pub async fn connect(chain_id: ChainId) -> Result<Self, ChainAccessError> {
        let url = chain_id.info().rpc_url();
        let provider = connect_tempo_url(url).await?;
        Ok(Self { provider, chain_id })
    }
}

#[async_trait]
impl ChainReader for TempoAdapter {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn native_balance(&self, address: Address) -> Result<U256, ChainAccessError> {
        self.provider
            .get_balance(address)
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn erc20_balance(
        &self,
        token: Address,
        owner: Address,
    ) -> Result<U256, ChainAccessError> {
        let calldata = erc20::balance_of_calldata(owner);
        let tx = TransactionRequest::default()
            .with_to(token)
            .with_input(calldata);
        let raw: Bytes = self.provider
            .raw_request("eth_call".into(), (tx, "latest"))
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))?;
        Ok(erc20::decode_u256_return(&raw))
    }

    async fn nonce(&self, address: Address) -> Result<u64, ChainAccessError> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<u64, ChainAccessError> {
        let gas: alloy::primitives::U64 = self.provider
            .raw_request("eth_estimateGas".into(), (tx, "latest"))
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))?;
        Ok(gas.to())
    }

    async fn gas_price(&self) -> Result<u128, ChainAccessError> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }

    async fn block_number(&self) -> Result<u64, ChainAccessError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| ChainAccessError::Rpc(e.to_string()))
    }
}
