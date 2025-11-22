use clap::{Args, Parser, Subcommand};

use crate::util::{parse_hex_or_dec_u16, parse_hex_or_dec_u8};

#[derive(Parser)]
#[command(
    name = "cap-locator-cli",
    version,
    about = "Control locator LEDs over USB HID"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// locatorの一覧を表示
    List(ListArgs),
    /// locatorのステータス(光っている/光っていない)を確認
    Status(StatusArgs),
    /// 指定locatorを光らせる
    On(SetArgs),
    /// 指定locatorの光を消す
    Off(SetArgs),
}

#[derive(Args, Clone, Debug)]
pub struct ListArgs {
    #[command(flatten)]
    pub filter: FilterArgs,
}

#[derive(Args, Clone, Debug)]
pub struct StatusArgs {
    /// locator id (シリアル番号 or HIDデバイスパスの部分文字列)。未指定ならフィルタに一致する全てを表示
    #[arg(long)]
    pub id: Option<String>,
    #[command(flatten)]
    pub filter: FilterArgs,
    #[command(flatten)]
    pub protocol: ProtocolArgs,
    /// ステータス応答の何バイト目をLED状態として解釈するか(0始まり)
    #[arg(long, default_value_t = 2)]
    pub status_index: usize,
}

#[derive(Args, Clone, Debug)]
pub struct SetArgs {
    /// locator id (シリアル番号 or HIDデバイスパスの部分文字列)
    #[arg(long)]
    pub id: String,
    #[command(flatten)]
    pub filter: FilterArgs,
    #[command(flatten)]
    pub protocol: ProtocolArgs,
    /// LED ONに送る値
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 1)]
    pub on_value: u8,
    /// LED OFFに送る値
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 0)]
    pub off_value: u8,
}

#[derive(Args, Clone, Debug)]
pub struct FilterArgs {
    /// vendor id (0x1234のような16進 or 10進)
    #[arg(long, value_parser = parse_hex_or_dec_u16)]
    pub vendor_id: Option<u16>,
    /// product id (0x5678のような16進 or 10進)
    #[arg(long, value_parser = parse_hex_or_dec_u16)]
    pub product_id: Option<u16>,
    /// HID usage pageでのフィルタ
    #[arg(long, value_parser = parse_hex_or_dec_u16)]
    pub usage_page: Option<u16>,
    /// HID usageでのフィルタ
    #[arg(long, value_parser = parse_hex_or_dec_u16)]
    pub usage: Option<u16>,
}

#[derive(Args, Clone, Debug)]
pub struct ProtocolArgs {
    /// 使用するreport id
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 0)]
    pub report_id: u8,
    /// feature reportの長さ(バイト)
    #[arg(long, default_value_t = 8)]
    pub report_len: usize,
    /// ステータス問い合わせで使うコマンド値
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 0x01)]
    pub command_status: u8,
    /// LED ON/OFFの指示で使うコマンド値
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 0x02)]
    pub command_set: u8,
}
