

use chain_access::{
    domain::chain::{Chain, RpcConfig},
    service::chain_router::ChainRouter,
};
use chain_access::adapters::tempo_handler::TempoHandler;
use chain_access::adapters::tempo_provider::{connect_tempo};
use crate::cli::Cli;

pub struct App {
    pub router: ChainRouter,
    pub rpc_config: Box<RpcConfig>,
}
impl App {
    pub async fn init(cli: &Cli) -> anyhow::Result<Self> {
        let mut router = ChainRouter::new();
        let rpc_config = Box::new(RpcConfig::new(cli.network, cli.chain));
        let provider = connect_tempo(&rpc_config).await?;
        // Arc handler (stub for now)
        router = router.register(Chain::Arc, Box::new(NotImplementedHandler));
        router = router.register(Chain::Tempo, Box::new(TempoHandler::new(provider)));

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