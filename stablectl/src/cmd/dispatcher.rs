use super::{chain_info, send, tokens, wallet};
use crate::app::App;
use crate::cli::{BalanceKind, ChainSubcmd, Cli, Command, SendKind, WalletSubcmd};
use crate::cmd::signer_builder::SignerConfig;
use chain_access::signer::SignerBackendType;

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
            WalletSubcmd::Create(_args) => wallet::create_wallet_privy().await,
            WalletSubcmd::Tokens => tokens::run_tokens().await,
            WalletSubcmd::Send(send_cmd) => match send_cmd.kind {
                SendKind::Native(args) => {
                    let signer_str = args.signer.signer;
                    let backend_type = signer_str
                        .parse::<SignerBackendType>()
                        .map_err(|_| anyhow::anyhow!("invalid --signer: {signer_str}"))?;
                    let signer_config = SignerConfig {
                        signer_backend: backend_type,
                        key_env: args.signer.key_env,
                        wallet_id: args.signer.wallet_id,
                    };
                    send::run_send_native(&app, &signer_config, &args.to, &args.amount).await
                }
                SendKind::Erc20(args) => {
                    let signer_str = args.signer.signer;
                    let backend_type = signer_str
                        .parse::<SignerBackendType>()
                        .map_err(|_| anyhow::anyhow!("invalid --signer: {signer_str}"))?;
                    let signer_config = SignerConfig {
                        signer_backend: backend_type,
                        key_env: args.signer.key_env,
                        wallet_id: args.signer.wallet_id,
                    };
                    send::run_send_erc20(
                        &app,
                        &signer_config,
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
