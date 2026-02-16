use anyhow::Context;

use chain_access::{
    domain::{
        actions::{Action, ExecCtx, ExecResult},
        account::AccountRef,
        chain::Chain,
    },
};

use crate::app::App;
use crate::cli::{ChainArg, NativeBalanceCmd};
use alloy_primitives::Address;
use std::str::FromStr;
pub async fn run(cmd: NativeBalanceCmd, app: &App) -> anyhow::Result<()> {
    let chain: Chain = match cmd.chain {
        ChainArg::Tempo => Chain::Tempo,
        ChainArg::Arc => Chain::Arc,
    };


    let owner: Address = Address::from_str(&cmd.owner)
        .map_err(|e| anyhow::anyhow!("invalid --owner address: {e}"))?;
    let owner_bytes: [u8; 20] = owner.into_array();

    // TODO: No account for now, still need to build account management utils.
    let ctx = ExecCtx::new(chain, app.rpc_config.clone(), None);

    let res = app.router
        .route(&ctx, Action::NativeBalance {  owner: owner_bytes })
        .await?;

    match res {
        ExecResult::Balance(b) => println!("{b}"),
        ExecResult::Submitted { tx_id } => println!("{tx_id}"),
    }

    Ok(())
}
