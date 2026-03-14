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
    /// Send tokens
    Send(SendCmd),
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

#[derive(Parser)]
pub struct SendCmd {
    #[command(subcommand)]
    pub kind: SendKind,
}

#[derive(Subcommand)]
pub enum SendKind {
    /// Send native TEMPO
    Native(SendNativeArgs),
    /// Send an ERC-20 token
    Erc20(SendErc20Args),
}

/// Common signer flags shared by all send commands.
#[derive(Parser)]
pub struct SignerArgs {
    /// Signer backend to use
    #[arg(long, default_value = "local-key")]
    pub signer: String,
    /// Environment variable containing the hex private key (for --signer local-key)
    #[arg(long)]
    pub key_env: String,
}

#[derive(Parser)]
pub struct SendNativeArgs {
    #[command(flatten)]
    pub signer: SignerArgs,
    /// Recipient address (0x...)
    #[arg(long)]
    pub to: String,
    /// Amount to send (e.g. "0.001")
    #[arg(long)]
    pub amount: String,
}

#[derive(Parser)]
pub struct SendErc20Args {
    #[command(flatten)]
    pub signer: SignerArgs,
    /// ERC-20 token contract address (0x...)
    #[arg(long)]
    pub token: String,
    /// Recipient address (0x...)
    #[arg(long)]
    pub to: String,
    /// Amount to send (e.g. "100.0")
    #[arg(long)]
    pub amount: String,
    /// Token decimals (default 18)
    #[arg(long, default_value_t = 18u8)]
    pub decimals: u8,
}
