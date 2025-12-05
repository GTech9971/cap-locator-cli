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
}

#[derive(Args, Clone, Debug)]
pub struct SetArgs {
    /// locator id (シリアル番号 or HIDデバイスパスの部分文字列)。未指定ならフィルタで1台に絞れないとエラー
    #[arg(long)]
    pub id: Option<String>,
    #[command(flatten)]
    pub filter: FilterArgs,
    #[command(flatten)]
    pub protocol: ProtocolArgs,
    /// RC2〜RC5/RA4のビットマスク(1=ON)でLED ON時に送る値
    #[arg(long, value_parser = parse_hex_or_dec_u8, default_value_t = 0x1f)]
    pub on_value: u8,
    /// RC2〜RC5/RA4のビットマスク(1=ON)でLED OFF時に送る値
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
    /// 入出力レポートの長さ(バイト)。不足分は0埋めで送信します
    #[arg(long, default_value_t = 64)]
    pub report_len: usize,
    /// 入力レポートを待つタイムアウト(ms)
    #[arg(long, default_value_t = 1000)]
    pub read_timeout_ms: i32,
}
