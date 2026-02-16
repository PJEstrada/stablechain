use async_trait::async_trait;

use crate::domain::actions::{Action, ExecCtx, ExecResult};

#[async_trait]
pub trait ChainHandler: Send + Sync {
    async fn execute(&self, ctx: &ExecCtx, action: Action) -> anyhow::Result<ExecResult>;
}