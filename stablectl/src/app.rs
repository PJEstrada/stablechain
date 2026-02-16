use anyhow::{Context, anyhow};

use chain_access::{
    adapters::{tempo_handler::TempoHandler, tempo_provider::connect_tempo},
    domain::chain::{Chain, RpcConfig},
    service::chain_router::ChainRouter,
};
use crate::cli::Cli;

pub struct App {
    pub router: ChainRouter,
    pub rpc_config: Box<RpcConfig>,
}
impl App {
    pub async fn init(cli: &Cli) -> anyhow::Result<Self> {
        let mut router = ChainRouter::new();
        let rpc_config = Box::new(RpcConfig::new(cli.network, cli.chain));

        // Arc handler (stub for now)
        router = router.register(Chain::Arc, Box::new(NotImplementedHandler));

        Ok(Self { router,  rpc_config})
    }
}

struct NotImplementedHandler;
#[async_trait::async_trait]
impl chain_access::service::chain_handlers::ChainHandler for NotImplementedHandler {
    async fn execute(
        &self,
        ctx: &chain_access::domain::actions::ExecCtx,
        _action: chain_access::domain::actions::Action,
    ) -> anyhow::Result<chain_access::domain::actions::ExecResult> {
        anyhow::bail!("chain '{}' not implemented yet", ctx.chain)
    }
}