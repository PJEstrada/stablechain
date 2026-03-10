use anyhow::Context;
use async_trait::async_trait;
use alloy::{
    primitives::{Address},
    providers::Provider,
};

use crate::{
    adapters::tempo_provider::TempoProvider,
    domain::actions::{Action, ExecCtx, ExecResult},
    service::chain_handlers::ChainHandler,
};


pub struct TempoHandler {
    provider: TempoProvider,
}

impl TempoHandler {
    pub fn new(provider: TempoProvider) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ChainHandler for TempoHandler {
    async fn execute(&self, _ctx: &ExecCtx, action: Action) -> anyhow::Result<ExecResult> {
        match action {
            Action::NativeBalance { owner } => {
                let owner = Address::from_slice(&owner);

                let balance = self
                    .provider
                    .get_balance(owner)
                    .await
                    .context("tempo get_balance failed")?;

                Ok(ExecResult::Balance(balance))
            }
            Action::Erc20Balance { .. } => {
                todo!("ERC20 not implemented yet")
            }
        }
    }
}
