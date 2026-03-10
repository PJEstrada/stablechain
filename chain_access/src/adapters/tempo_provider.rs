// chain_access/src/adapters/tempo_provider.rs

use alloy::providers::fillers::{FillProvider, JoinFill, NonceFiller};
use anyhow::Context;
use alloy::providers::{Identity, ProviderBuilder, RootProvider};
use tempo_alloy::{TempoFillers, TempoNetwork};

use crate::domain::chain::RpcConfig;

pub type TempoProvider = FillProvider<
    JoinFill<Identity, TempoFillers<NonceFiller>>,
    RootProvider<TempoNetwork>,
    TempoNetwork
>;
pub async fn connect_tempo(cfg: &RpcConfig) -> anyhow::Result<TempoProvider> {
    let provider = ProviderBuilder::new_with_network::<TempoNetwork>()
        .connect(&cfg.url())
        .await
        .with_context(|| format!("failed to connect to Tempo RPC at {}", cfg.url()))?;

    Ok(provider)
}
