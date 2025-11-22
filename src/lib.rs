pub mod cli;
pub mod commands;
pub mod env_config;
pub mod hid;
pub mod util;

pub use cli::{
    Cli, Commands, FilterArgs, ListArgs, ProtocolArgs, SetArgs, StatusArgs,
};
pub use commands::{handle_list, handle_set, handle_status};
pub use env_config::{load_env_defaults, merge_filter, EnvDefaults};
pub use util::{format_bytes, format_usage, parse_hex_or_dec_u16, parse_hex_or_dec_u8};

#[cfg(test)]
mod tests;
