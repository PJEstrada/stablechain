use crate::app::App;
use crate::cli::{BalanceKind, ChainSubcmd, Cli, Command, SendKind, WalletSubcmd};

use super::{chain_info, send, wallet};

pub async fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let app = App::init(cli.chain).await?;

    match cli.command {
        Command::Chain(cmd) => match cmd.sub {
            ChainSubcmd::Info => chain_info::run_info(&app).await,
        },
        Command::Wallet(cmd) => match cmd.sub {
            WalletSubcmd::Balance(balance_cmd) => match balance_cmd.kind {
                BalanceKind::Native(args) => wallet::run_balance_native(&app, &args.address).await,
                BalanceKind::Erc20(args) => {
                    wallet::run_balance_erc20(&app, &args.token, &args.address, args.decimals).await
                }
            },
            WalletSubcmd::Send(send_cmd) => match send_cmd.kind {
                SendKind::Native(args) => {
                    send::run_send_native(&app, &args.signer.key_env, &args.to, &args.amount).await
                }
                SendKind::Erc20(args) => {
                    send::run_send_erc20(
                        &app,
                        &args.signer.key_env,
                        &args.token,
                        &args.to,
                        &args.amount,
                        args.decimals,
                    )
                    .await
                }
            },
        },
    }
}
