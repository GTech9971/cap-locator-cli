# cap-locator-cli

USB HID経由で「locator」を制御するRust製コマンドラインツールです。Cap Locator のPIC16F1455向けHIDファームウェア(Endpoint1 IN/OUT 64B)と通信できるようデフォルト値を合わせています。Feature Reportは使わず、Interrupt IN/OUTの先頭バイトでコマンドをやり取りします。

## 前提

- Rust がインストールされていること
- OS側で `hidapi` を利用できること（macOSなら `brew install hidapi` など）

## ビルド

```bash
cargo build
```

## 使い方

基本はサブコマンド形式です。ベンダーID/プロダクトIDは16進または10進で指定できます。

### .env でデフォルトを指定する

`.env` に以下のように書くと、CLI引数を省略しても自動で補われます（CLIで指定した値が優先されます）。リポジトリ同梱の`.env`はCap LocatorファームウェアのVID/PID/Usageに合わせてあります。

```
VENDOR_ID=0x04d8
PRODUCT_ID=0x1455
USAGE_PAGE=0xFF00
USAGE=0x0001
```

### locator一覧

```bash
cargo run -- list
```

### ステータス確認（LEDが光っているか）

```bash
cargo run -- status
```

フィルタに一致するlocatorをすべて表示します。

### LED点灯/消灯

```bash
# 点灯 (RC2〜RC5/RA4を全て点灯させるマスク0x1fがデフォルト)
cargo run -- on

# 消灯
cargo run -- off
```

フィルタに合致するデバイスが1台ならそれを対象にします。複数見つかった場合はエラーになるので、`--vendor-id` / `--product-id` / `--usage-page` / `--usage` で絞り込んでください。`--on-value` / `--off-value` でRC2〜RC5/RA4のビットマスクを指定できます（デフォルトは0x1f/0x00）。

## オプション早見表

- `--vendor-id`, `--product-id` : ベンダー/プロダクトでフィルタ (Cap Locatorは 0x04d8 / 0x1455)
- `--usage-page`, `--usage` : HID Usageでフィルタ (Cap Locatorは 0xFF00 / 0x0001)
- `--report-len` : IN/OUTレポート長（デフォルト64バイト。足りない分は0埋め）
- `--read-timeout-ms` : ステータス取得時に入力レポートを待つ時間 (デフォルト1000ms)
- `--on-value` / `--off-value` : 点灯/消灯指示で送るLEDマスク (デフォルト0x1f / 0x00)

## プロトコルについて

Feature Reportは使用せず、IN/OUTレポートのみでコマンドをやり取りします。パケット長は `report-len` で決め、足りない分は0埋めになります。

- ステータス取得: `[0x01, 0x00, ...]` を OUT 送信。応答は `[0xff, led_mask, ...]` で、`led_mask` の下位5bitが RC2〜RC5/RA4 のON/OFFを表します（0なら全消灯と判定）。
- 点灯/消灯: `[0x02, led_mask, ...]` を OUT 送信。`led_mask` の各ビットで RC2〜RC5/RA4 を1=ON/0=OFF とします。`on` は `--on-value`、`off` は `--off-value` のマスクを送ります。
- いずれの応答も先頭バイトは常に `0xff` が返る想定です。

## テスト

```bash
cargo test
```
