use std::str::FromStr;

use alloy::network::TransactionBuilder;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use comfy_table::{Cell, Table, presets};
use console::style;

use crate::app::App;
use crate::cmd::signer_builder::{SignerConfig, build_signer};
use crate::display::spinner;
use chain_access::adapters::connect_writer;

pub async fn run_send_native(
    _app: &App,
    _signer_config: &SignerConfig,
    _to: &str,
    _amount: &str,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "Sending native TEMPO tokens is not supported.\n\
        Tempo has no native gas token. Instead, send TIP-20 tokens (stablecoins) like:\n\
        - pathUSD\n\
        - AlphaUSD\n\
        - BetaUSD\n\
        - ThetaUSD\n\
        \n\
        Get faucet funds at: https://docs.tempo.xyz/quickstart/faucet\n\
        \n\
        Example:\n\
        cargo run -- wallet send erc20 --token <TOKEN_CONTRACT> --to <ADDRESS> --amount <AMOUNT> --decimals 18"
    )
}

pub async fn run_send_erc20(
    app: &App,
    signer_config: &SignerConfig,
    token: &str,
    to: &str,
    amount: &str,
    decimals: u8,
) -> anyhow::Result<()> {
    let signer = build_signer(signer_config)?;
    let from = signer.address().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let token_addr =
        Address::from_str(token).map_err(|e| anyhow::anyhow!("invalid --token: {e}"))?;
    let to_addr = Address::from_str(to).map_err(|e| anyhow::anyhow!("invalid --to: {e}"))?;
    let amount_u256: U256 = parse_units(amount, decimals)
        .map_err(|e| anyhow::anyhow!("invalid --amount: {e}"))?
        .into();

    let chain_id = app.reader.chain_id();
    let calldata = chain_access::domain::erc20::transfer_calldata(to_addr, amount_u256);

    let pb = spinner("Preparing transaction...");
    let nonce = app
        .reader
        .nonce(from)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let gas_price = app
        .reader
        .gas_price()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let tx = TransactionRequest::default()
        .with_from(from)
        .with_to(token_addr)
        .with_input(calldata)
        .with_nonce(nonce)
        .with_chain_id(chain_id.info().chain_id());

    let gas = app
        .reader
        .estimate_gas(&tx)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let full_tx = tx
        .with_gas_limit(gas)
        .with_max_fee_per_gas(gas_price)
        .with_max_priority_fee_per_gas(0);

    pb.set_message("Signing...");
    let raw = signer
        .sign_transaction(full_tx)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    pb.set_message("Broadcasting...");
    let writer = connect_writer(chain_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let tx_hash = writer
        .send_raw_transaction(raw)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    pb.set_message("Waiting for confirmation...");
    let receipt = writer
        .wait_for_receipt(&tx_hash)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    // Display transaction result in a table
    let key = |s: &str| Cell::new(s).fg(comfy_table::Color::DarkGrey);

    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![
        key("Signer"),
        Cell::new(format!("{} {}", signer.signer_kind(), from)),
    ]);
    table.add_row(vec![key("Token"), Cell::new(token_addr.to_string())]);
    table.add_row(vec![key("To"), Cell::new(to_addr.to_string())]);
    table.add_row(vec![key("Amount"), Cell::new(format!("{} tokens", amount))]);

    println!("{table}");
    println!();

    let status = if receipt.status() {
        style("✔ Confirmed").green()
    } else {
        style("✗ Failed").red()
    };

    println!("{} in block #{}", status, receipt.block_number.unwrap_or(0));
    println!(
        "  {} {}",
        style("Tx hash").dim(),
        style(format!("{tx_hash}")).cyan()
    );
    println!(
        "  {} {}",
        style("Explorer").dim(),
        style(format!("https://explore.testnet.tempo.xyz/tx/{tx_hash}")).cyan()
    );
    Ok(())
}
