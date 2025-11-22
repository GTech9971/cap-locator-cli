use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use hidapi::HidApi;

use cap_locator_cli::{
    handle_list, handle_set, handle_status, load_env_defaults, Cli, Commands,
};

fn main() -> Result<()> {
    // .envでデフォルトのベンダーIDなどを読み込み
    dotenv().ok();

    let cli = Cli::parse();
    let env_defaults = load_env_defaults()?;
    let api = HidApi::new().context("failed to initialize HID API")?;

    match cli.command {
        Commands::List(args) => handle_list(&api, &args, &env_defaults),
        Commands::Status(args) => handle_status(&api, &args, &env_defaults),
        Commands::On(args) => handle_set(&api, &args, &env_defaults, true),
        Commands::Off(args) => handle_set(&api, &args, &env_defaults, false),
    }
}
