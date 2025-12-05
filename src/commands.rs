use anyhow::{Context, Result, bail};
use hidapi::HidApi;

use crate::cli::{ListArgs, SetArgs, StatusArgs};
use crate::env_config::{EnvDefaults, merge_filter};
use crate::hid::{pick_single_device, query_status, set_light, snapshot_devices};
use crate::util::{format_bytes, format_usage};

/// locator一覧をフィルタ付きで表示する
///
/// - .envとCLI引数をマージして対象デバイスを抽出
/// - 見つからなければその旨を標準出力に表示
pub fn handle_list(api: &HidApi, args: &ListArgs, env: &EnvDefaults) -> Result<()> {
    let filter = merge_filter(&args.filter, env);
    let devices = snapshot_devices(api, &filter);
    if devices.is_empty() {
        println!("locatorは見つかりませんでした");
        return Ok(());
    }

    for device in devices {
        let locator_id = device.locator_id();
        let serial = device
            .serial_number
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let path = device.path.to_string_lossy();
        println!(
            "id={:<20} serial={:<20} vendor=0x{:04x} product=0x{:04x} usage={} path={}",
            locator_id,
            serial,
            device.vendor_id,
            device.product_id,
            format_usage(device.usage_page, device.usage),
            path
        );
    }
    Ok(())
}

/// 対象locatorのLED点灯状態を問い合わせて表示する
///
/// - .env/CLIのフィルタでデバイスを絞り込む
/// - Output Reportでステータスコマンドを送信し、応答(先頭0xff)のLEDマスクで判定
pub fn handle_status(api: &HidApi, args: &StatusArgs, env: &EnvDefaults) -> Result<()> {
    let filter = merge_filter(&args.filter, env);
    let mut devices = snapshot_devices(api, &filter);
    if let Some(id) = args.id.as_deref().filter(|s| !s.is_empty()) {
        devices.retain(|d| d.matches_id(id));
    }

    if devices.is_empty() {
        bail!("対象となるlocatorが見つかりませんでした");
    }

    for device in devices {
        let locator_id = device.locator_id();
        let handle = api
            .open_path(device.path.as_c_str())
            .with_context(|| format!("open device {}", locator_id))?;

        let status =
            query_status(&handle, &args.protocol).with_context(|| {
                format!(
                    "ステータス取得に失敗しました (id={})",
                    locator_id
                )
            })?;

        println!(
            "id={:<20} status={} mask=0x{mask:02x} raw=[{raw}]",
            locator_id,
            if status.is_on { "on " } else { "off" },
            mask = status.mask,
            raw = format_bytes(&status.raw)
        );
    }

    Ok(())
}

/// 単一のlocatorに対してLED点灯/消灯コマンドを送信する
///
/// - .env/CLIのフィルタでデバイスを検索し、1件に絞れないとエラー
/// - Output ReportでON/OFF値を送信し、成功したら結果を表示
pub fn handle_set(api: &HidApi, args: &SetArgs, env: &EnvDefaults, turn_on: bool) -> Result<()> {
    let filter = merge_filter(&args.filter, env);
    let device = pick_single_device(api, &filter, args.id.as_deref())?;
    let locator_id = device.locator_id();
    let handle = api
        .open_path(device.path.as_c_str())
        .with_context(|| format!("open device {}", locator_id))?;

    set_light(
        &handle,
        &args.protocol,
        turn_on,
        args.on_value,
        args.off_value,
    )
    .with_context(|| {
        format!(
            "LED制御に失敗しました (id={})",
            locator_id
        )
    })?;

    let status = query_status(&handle, &args.protocol).with_context(|| {
        format!(
            "ステータス取得に失敗しました (id={})",
            locator_id
        )
    })?;

    println!(
        "id={:<20} status={} mask=0x{mask:02x} raw=[{raw}]",
        locator_id,
        if status.is_on { "on " } else { "off" },
        mask = status.mask,
        raw = format_bytes(&status.raw)
    );
    Ok(())
}
