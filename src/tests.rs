use crate::cli::{FilterArgs, ProtocolArgs};
use crate::env_config::{merge_filter, EnvDefaults};
use crate::hid::{mock::MockDevice, query_status, set_light};
use crate::util::{format_bytes, format_usage, parse_hex_or_dec_u16, parse_hex_or_dec_u8};

// 数値パーサが16進/10進を正しく受け付けることを確認
#[test]
fn parse_hex_or_dec_u16_accepts_hex() {
    assert_eq!(parse_hex_or_dec_u16("0x10").unwrap(), 0x10);
    assert_eq!(parse_hex_or_dec_u16("0Xff").unwrap(), 0xff);
}

#[test]
fn parse_hex_or_dec_u8_accepts_dec() {
    assert_eq!(parse_hex_or_dec_u8("15").unwrap(), 15);
}

// 各種フォーマッタが期待通りの文字列を返すことを確認
#[test]
fn format_usage_outputs_dashes() {
    assert_eq!(format_usage(None, None), "-");
    assert_eq!(format_usage(Some(0x100), None), "0x0100:-");
    assert_eq!(format_usage(None, Some(0x200)), "-:0x0200");
}

#[test]
fn format_bytes_outputs_space_separated_hex() {
    assert_eq!(format_bytes(&[0, 1, 0x10, 0xff]), "00 01 10 ff");
}

// フィルタ統合ロジックがCLI優先で環境変数を補完する挙動を確認
#[test]
fn merge_filter_prefers_cli_over_env_defaults() {
    let cli = FilterArgs {
        vendor_id: Some(0x1234),
        product_id: None,
        usage_page: Some(0x01),
        usage: None,
    };
    let env = EnvDefaults {
        vendor_id: Some(0x9999),
        product_id: Some(0x7777),
        usage_page: None,
        usage: Some(0x02),
    };

    let merged = merge_filter(&cli, &env);
    assert_eq!(merged.vendor_id, Some(0x1234));
    assert_eq!(merged.product_id, Some(0x7777));
    assert_eq!(merged.usage_page, Some(0x01));
    assert_eq!(merged.usage, Some(0x02));
}

#[test]
fn merge_filter_fills_missing_from_env() {
    let cli = FilterArgs {
        vendor_id: None,
        product_id: None,
        usage_page: None,
        usage: None,
    };
    let env = EnvDefaults {
        vendor_id: Some(1),
        product_id: Some(2),
        usage_page: Some(3),
        usage: Some(4),
    };

    let merged = merge_filter(&cli, &env);
    assert_eq!(merged.vendor_id, Some(1));
    assert_eq!(merged.product_id, Some(2));
    assert_eq!(merged.usage_page, Some(3));
    assert_eq!(merged.usage, Some(4));
}

// HID通信ロジックのモックテスト
#[test]
fn query_status_uses_status_index_and_records_command() {
    let device = MockDevice::with_response(vec![0x05, 0x01, 0x99, 0x00]);
    let protocol = ProtocolArgs {
        report_id: 0x05,
        report_len: 4,
        command_status: 0x10,
        command_set: 0x20,
    };

    let status = query_status(&device, &protocol, 2).unwrap();
    assert!(status.is_on);
    assert_eq!(device.last_sent().unwrap(), vec![0x05, 0x10, 0x00, 0x00]);
    assert_eq!(status.raw, vec![0x05, 0x01, 0x99, 0x00]);
}

#[test]
fn set_light_sends_on_and_off_commands() {
    let device = MockDevice::with_response(vec![0x00]);
    let protocol = ProtocolArgs {
        report_id: 0x07,
        report_len: 4,
        command_status: 0x01,
        command_set: 0xAA,
    };

    set_light(&device, &protocol, true, 0x11, 0x22).unwrap();
    assert_eq!(device.last_sent().unwrap(), vec![0x07, 0xAA, 0x11, 0x00]);

    set_light(&device, &protocol, false, 0x11, 0x22).unwrap();
    assert_eq!(device.last_sent().unwrap(), vec![0x07, 0xAA, 0x22, 0x00]);
}
