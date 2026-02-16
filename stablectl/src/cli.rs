use clap::{Parser, Subcommand, ValueEnum};
use chain_access::domain::chain::Chain;
use chain_access::domain::network::Network;

#[derive(Parser)]
#[command(name = "stablectl")]
#[command(about = "Stablecoin payroll system CLI (POC)")]
#[command(version, long_about = None, propagate_version = true)]
pub struct Cli {
    /// Chain family (either arc or tempo)
    #[arg(long, global = true, default_value_t = Chain::Tempo)]
    pub chain: Chain,

    /// Network (mainnet or testnet)
    #[arg(long, global = true, default_value_t = Network::Testnet)]
    pub network: Network,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Balance(NativeBalanceCmd),
}

#[derive(Parser)]
pub struct NativeBalanceCmd {
    #[arg(long, value_enum)]
    pub chain: ChainArg,
    #[arg(long)]
    pub owner: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ChainArg {
    Tempo,
    Arc,
}
