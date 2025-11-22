use std::env;

use anyhow::{anyhow, Result};

use crate::cli::FilterArgs;
use crate::util::parse_hex_or_dec_u16;

#[derive(Clone, Debug, Default)]
pub struct EnvDefaults {
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

pub fn load_env_defaults() -> Result<EnvDefaults> {
    Ok(EnvDefaults {
        vendor_id: read_env_u16("VENDOR_ID")?,
        product_id: read_env_u16("PRODUCT_ID")?,
        usage_page: read_env_u16("USAGE_PAGE")?,
        usage: read_env_u16("USAGE")?,
    })
}

/// CLI指定を優先し、欠けている項目だけ環境変数から補う
pub fn merge_filter(cli: &FilterArgs, env: &EnvDefaults) -> FilterArgs {
    FilterArgs {
        vendor_id: cli.vendor_id.or(env.vendor_id),
        product_id: cli.product_id.or(env.product_id),
        usage_page: cli.usage_page.or(env.usage_page),
        usage: cli.usage.or(env.usage),
    }
}

fn read_env_u16(key: &str) -> Result<Option<u16>> {
    match env::var(key) {
        Ok(value) => parse_hex_or_dec_u16(&value)
            .map(Some)
            .map_err(|e| anyhow!("{} の値を解釈できません: {}", key, e)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(anyhow!("{} が非Unicodeのため読み取れません", key))
        }
    }
}
