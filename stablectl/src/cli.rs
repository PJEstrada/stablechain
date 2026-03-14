use clap::{Parser, Subcommand};
use chain_access::domain::chain_id::ChainId;

#[derive(Parser)]
#[command(name = "stablectl")]
#[command(about = "Signer-agnostic transfers on the Tempo testnet")]
#[command(version)]
pub struct Cli {
    /// Chain to connect to
    #[arg(long, global = true, default_value = "tempo-testnet")]
    pub chain: ChainId,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Chain metadata and current block number
    Chain(ChainCmd),
    /// Wallet operations
    Wallet(WalletCmd),
}

#[derive(Parser)]
pub struct ChainCmd {
    #[command(subcommand)]
    pub sub: ChainSubcmd,
}

#[derive(Subcommand)]
pub enum ChainSubcmd {
    /// Print chain info and current block
    Info,
}

#[derive(Parser)]
pub struct WalletCmd {
    #[command(subcommand)]
    pub sub: WalletSubcmd,
}

#[derive(Subcommand)]
pub enum WalletSubcmd {
    /// Query account balance
    Balance(BalanceCmd),
}

#[derive(Parser)]
pub struct BalanceCmd {
    #[command(subcommand)]
    pub kind: BalanceKind,
}

#[derive(Subcommand)]
pub enum BalanceKind {
    /// Native token balance
    Native(NativeBalanceArgs),
    /// ERC-20 token balance
    Erc20(Erc20BalanceArgs),
}

#[derive(Parser)]
pub struct NativeBalanceArgs {
    /// Wallet address (0x...)
    #[arg(long)]
    pub address: String,
}

#[derive(Parser)]
pub struct Erc20BalanceArgs {
    /// ERC-20 token contract address (0x...)
    #[arg(long)]
    pub token: String,
    /// Wallet address (0x...)
    #[arg(long)]
    pub address: String,
    /// Token decimals for display
    #[arg(long, default_value_t = 18u8)]
    pub decimals: u8,
}
