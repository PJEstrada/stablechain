use std::collections::HashMap;

use crate::domain::chain::Chain;
use crate::domain::actions::{Action, ExecCtx, ExecResult};
use crate::service::chain_handlers::ChainHandler;

pub struct ChainRouter {
    handlers: HashMap<Chain, Box<dyn ChainHandler>>,
}

impl ChainRouter {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(mut self, chain: Chain, handler: Box<dyn ChainHandler>) -> Self {
        self.handlers.insert(chain, handler);
        self
    }

    pub async fn route(&self, ctx: &ExecCtx, action: Action) -> anyhow::Result<ExecResult> {
        let h = self.handlers.get(&ctx.chain)
            .ok_or_else(|| anyhow::anyhow!("no handler registered for chain {}", ctx.chain))?;
        h.execute(ctx, action).await
    }
}