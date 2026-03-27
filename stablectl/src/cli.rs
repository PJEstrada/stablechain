use chain_access::domain::chain_id::ChainId;
use clap::{Parser, Subcommand};

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
    /// Signer session/auth operations
    Signer(SignerCmd),
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
    /// Create a new wallet
    Create(CreateWalletArgs),
    /// List supported TIP-20 tokens
    Tokens,
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

#[derive(Parser)]
pub struct CreateWalletArgs {
    #[arg(long, default_value = "")]
    pub owner: String,
}

#[derive(Parser)]
pub struct SignerCmd {
    #[command(subcommand)]
    pub sub: SignerSubcmd,
}

#[derive(Subcommand)]
pub enum SignerSubcmd {
    /// Save user JWT session for --signer privy-user
    Login(SignerLoginArgs),
    /// Open local browser flow to capture and save a user JWT
    LoginBrowser(SignerLoginBrowserArgs),
    /// Remove local user JWT session
    Logout,
    /// Show current signer session status
    Whoami,
}

#[derive(Parser)]
pub struct SignerLoginArgs {
    /// User JWT token to store locally for --signer privy-user
    #[arg(long)]
    pub jwt: String,
}

#[derive(Parser)]
pub struct SignerLoginBrowserArgs {
    /// Local port for callback/capture page
    #[arg(long, default_value_t = 8787)]
    pub port: u16,
}

#[derive(Subcommand)]
pub enum SendKind {
    /// Send native TEMPO (NOT SUPPORTED - Tempo has no native token)
    Native(SendNativeArgs),
    /// Send a TIP-20/ERC-20 token (e.g., pathUSD, AlphaUSD, BetaUSD, ThetaUSD)
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
    pub key_env: Option<String>,

    /// Wallet ID (for --signer privy or --signer privy-user)
    #[arg(long, default_value = "")]
    pub wallet_id: Option<String>,
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
