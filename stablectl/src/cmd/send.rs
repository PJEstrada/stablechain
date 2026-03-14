use std::str::FromStr;

use alloy::network::TransactionBuilder;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use console::style;

use chain_access::adapters::connect_writer;
use chain_access::signer::{LocalKeySigner, SignerBackend};

use crate::app::App;
use crate::display::spinner;

pub async fn run_send_native(
    app: &App,
    key_env: &str,
    to: &str,
    amount: &str,
) -> anyhow::Result<()> {
    let signer = LocalKeySigner::from_env(key_env).map_err(|e| anyhow::anyhow!("{e}"))?;
    let from = signer.address().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let to_addr = Address::from_str(to).map_err(|e| anyhow::anyhow!("invalid --to: {e}"))?;
    let value: U256 = parse_units(amount, 18u8)
        .map_err(|e| anyhow::anyhow!("invalid --amount: {e}"))?
        .into();

    let chain_id = app.reader.chain_id();

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
        .with_to(to_addr)
        .with_value(value)
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

    let status = if receipt.status() {
        style("confirmed").green()
    } else {
        style("failed").red()
    };
    println!(
        "{} {} {}",
        style("tx").dim(),
        style(format!("{tx_hash}")).cyan(),
        status.bold()
    );
    Ok(())
}

pub async fn run_send_erc20(
    app: &App,
    key_env: &str,
    token: &str,
    to: &str,
    amount: &str,
    decimals: u8,
) -> anyhow::Result<()> {
    let signer = LocalKeySigner::from_env(key_env).map_err(|e| anyhow::anyhow!("{e}"))?;
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

    let status = if receipt.status() {
        style("confirmed").green()
    } else {
        style("failed").red()
    };
    println!(
        "{} {} {}",
        style("tx").dim(),
        style(format!("{tx_hash}")).cyan(),
        status.bold()
    );
    Ok(())
}
