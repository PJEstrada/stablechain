use crate::cli::{Cli, Command};
use crate::app::App;

pub mod balance;

pub async fn dispatch(cli: Cli) -> anyhow::Result<()> {
    // Initialize the app once with the CLI config
    let app = App::init(&cli).await?;

    // Dispatch to the appropriate command, passing the app
    match cli.command {
        Command::Balance(cmd) => balance::run(cmd, &app).await,
    }
}