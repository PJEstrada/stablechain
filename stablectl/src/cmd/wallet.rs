use privy_rs::PrivyClient;
use std::str::FromStr;

use crate::app::App;
use crate::display::spinner;
use alloy::primitives::Address;
use alloy::primitives::utils::format_units;
use chain_access::ports::privy::{APP_ID_ENV_VAR, APP_SECRET_ENV_VAR};
use chain_access::ports::privy::{WalletService, WalletsManager};
use comfy_table::{Cell, Color, Table, presets};
use console::style;
use privy_rs::generated::types::{CreateWalletBody, WalletChainType};

pub async fn run_balance_native(app: &App, address: &str) -> anyhow::Result<()> {
    let addr = Address::from_str(address).map_err(|e| anyhow::anyhow!("invalid --address: {e}"))?;

    let pb = spinner("Fetching native balance...");
    let balance = app
        .reader
        .native_balance(addr)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    let formatted =
        format_units(balance, 18u8).map_err(|e| anyhow::anyhow!("format error: {e}"))?;
    println!(
        "{} {}",
        style(formatted).bold().green(),
        style("TEMPO").dim()
    );
    Ok(())
}

pub async fn run_balance_erc20(
    app: &App,
    token: &str,
    address: &str,
    decimals: u8,
) -> anyhow::Result<()> {
    let token_addr =
        Address::from_str(token).map_err(|e| anyhow::anyhow!("invalid --token: {e}"))?;
    let owner_addr =
        Address::from_str(address).map_err(|e| anyhow::anyhow!("invalid --address: {e}"))?;

    let pb = spinner("Fetching ERC-20 balance...");
    let balance = app
        .reader
        .erc20_balance(token_addr, owner_addr)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    let formatted =
        format_units(balance, decimals).map_err(|e| anyhow::anyhow!("format error: {e}"))?;
    println!("{}", style(formatted).bold().green());
    Ok(())
}

pub async fn create_wallet_privy() -> anyhow::Result<()> {
    let _app_id = std::env::var(APP_ID_ENV_VAR)
        .map_err(|_| anyhow::anyhow!("missing env var: {APP_ID_ENV_VAR}"))?;
    let _app_secret = std::env::var(APP_SECRET_ENV_VAR)
        .map_err(|_| anyhow::anyhow!("missing env var: {APP_SECRET_ENV_VAR}"))?;
    let client = PrivyClient::new_from_env()?;
    let mgr = WalletsManager::new(client);

    // create a simple wallet - server ownerd
    let body = CreateWalletBody {
        chain_type: WalletChainType::Ethereum,
        additional_signers: None,
        owner: None,
        owner_id: None,
        policy_ids: vec![],
    };
    let wallet = mgr.create_wallet(body).await?;

    let key = |s: &str| Cell::new(s).fg(Color::DarkGrey);

    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![key("Wallet ID"), Cell::new(wallet.id)]);
    table.add_row(vec![key("Wallet Address"), Cell::new(wallet.address)]);
    table.add_row(vec![
        key("Owner ID"),
        Cell::new(wallet.owner_id.unwrap_or_else(|| "None".to_string())),
    ]);
    table.add_row(vec![
        key("Public Key"),
        Cell::new(wallet.public_key.unwrap_or_else(|| "None".to_string())),
    ]);

    println!("{table}");
    Ok(())
}
