use std::str::FromStr;

use alloy::primitives::Address;
use alloy::primitives::utils::format_units;
use console::style;

use crate::app::App;
use crate::display::spinner;

pub async fn run_balance_native(app: &App, address: &str) -> anyhow::Result<()> {
    let addr = Address::from_str(address)
        .map_err(|e| anyhow::anyhow!("invalid --address: {e}"))?;

    let pb = spinner("Fetching native balance...");
    let balance = app.reader.native_balance(addr).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    let formatted = format_units(balance, 18u8)
        .map_err(|e| anyhow::anyhow!("format error: {e}"))?;
    println!("{} {}", style(formatted).bold().green(), style("TEMPO").dim());
    Ok(())
}

pub async fn run_balance_erc20(
    app: &App,
    token: &str,
    address: &str,
    decimals: u8,
) -> anyhow::Result<()> {
    let token_addr = Address::from_str(token)
        .map_err(|e| anyhow::anyhow!("invalid --token: {e}"))?;
    let owner_addr = Address::from_str(address)
        .map_err(|e| anyhow::anyhow!("invalid --address: {e}"))?;

    let pb = spinner("Fetching ERC-20 balance...");
    let balance = app.reader.erc20_balance(token_addr, owner_addr).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    let formatted = format_units(balance, decimals)
        .map_err(|e| anyhow::anyhow!("format error: {e}"))?;
    println!("{}", style(formatted).bold().green());
    Ok(())
}
