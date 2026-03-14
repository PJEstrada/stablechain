use comfy_table::{Cell, Color, Table, presets};

use crate::app::App;
use crate::display::spinner;

pub async fn run_info(app: &App) -> anyhow::Result<()> {
    let id = app.reader.chain_id();
    let info = id.info();

    let pb = spinner("Fetching block number...");
    let block = app
        .reader
        .block_number()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    pb.finish_and_clear();

    let key = |s: &str| Cell::new(s).fg(Color::DarkGrey);

    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![key("Chain"), Cell::new(info.name())]);
    table.add_row(vec![
        key("Chain ID"),
        Cell::new(info.chain_id().to_string()),
    ]);
    table.add_row(vec![key("RPC"), Cell::new(info.rpc_url())]);
    table.add_row(vec![key("Explorer"), Cell::new(info.explorer_url())]);
    table.add_row(vec![key("Block"), Cell::new(format!("#{block}"))]);

    println!("{table}");
    Ok(())
}
