/// Tempo testnet TIP-20 token addresses
pub const PATHUSD: &str = "0x20c0000000000000000000000000000000000000";
pub const ALPHAUSD: &str = "0x20c0000000000000000000000000000000000001";
pub const BETAUSD: &str = "0x20c0000000000000000000000000000000000002";
pub const THETAUSD: &str = "0x20c0000000000000000000000000000000000003";

/// All supported TIP-20 tokens on Tempo testnet
pub const SUPPORTED_TOKENS: &[(&str, &str)] = &[
    ("pathUSD", PATHUSD),
    ("AlphaUSD", ALPHAUSD),
    ("BetaUSD", BETAUSD),
    ("ThetaUSD", THETAUSD),
];

use comfy_table::{Table, presets, Cell};
use console::style;

pub async fn run_tokens() -> anyhow::Result<()> {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_header(vec!["Token", "Contract Address"]);
    
    for (symbol, address) in SUPPORTED_TOKENS {
        table.add_row(vec![
            Cell::new(symbol),
            Cell::new(address),
        ]);
    }
    
    println!("{}", style("Supported TIP-20 tokens on Tempo testnet:").bold());
    println!();
    println!("{}", table);
    println!();
    println!("Get faucet funds at: {}", style("https://docs.tempo.xyz/quickstart/faucet").cyan());
    println!();
    println!("Example usage:");
    println!("  cargo run -- wallet send erc20 \\");
    println!("    --token {} \\", ALPHAUSD);
    println!("    --to 0x4FE210d1A43D3D49D636cB33E142223a493885E1 \\");
    println!("    --amount 1000 \\");
    println!("    --decimals 18 \\");
    println!("    --signer privy \\");
    println!("    --wallet-id <WALLET_ID>");
    
    Ok(())
}
