# cap-locator-cli

USB HID経由で「locator」を制御するRust製コマンドラインツールです。locatorの一覧取得、LEDの点灯状態確認、点灯/消灯指示をサポートします。

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

`.env` に以下のように書くと、CLI引数を省略しても自動で補われます（CLIで指定した値が優先されます）。

```
VENDOR_ID=0x1234
PRODUCT_ID=0x5678
USAGE_PAGE=0xFF00
USAGE=0x0001
```

### locator一覧

```bash
cargo run -- list --vendor-id 0x1234 --product-id 0x5678
```

### ステータス確認（LEDが光っているか）

```bash
cargo run -- status --vendor-id 0x1234 --product-id 0x5678 --report-id 0 --command-status 1 --status-index 2
```

`--id <locator-id>` を付けると特定のlocatorだけ確認します。`locator-id` はシリアル番号があればそれを、なければHIDパスを参照します（部分一致も可）。

### LED点灯/消灯

```bash
# 点灯
cargo run -- on  --id ABC123 --vendor-id 0x1234 --product-id 0x5678 --report-id 0 --command-set 2 --on-value 1

# 消灯
cargo run -- off --id ABC123 --vendor-id 0x1234 --product-id 0x5678 --report-id 0 --command-set 2 --off-value 0
```

`--on-value` / `--off-value` でデバイスが期待する値を指定できます（デフォルトは1/0）。

## オプション早見表

- `--vendor-id`, `--product-id` : ベンダー/プロダクトでフィルタ
- `--usage-page`, `--usage` : HID Usageでフィルタ
- `--report-id` : Feature Reportに使うReport ID（デフォルト0）
- `--report-len` : Feature Report長（デフォルト8バイト）
- `--command-status` : ステータス問い合わせに使うコマンド値（デフォルト0x01）
- `--command-set` : LED制御に使うコマンド値（デフォルト0x02）
- `--status-index` : 応答中どのバイトをLED状態として読むか（デフォルト2。0以外を「点灯」とみなします）
- `--on-value` / `--off-value` : 点灯/消灯指示で送る値

## プロトコルについて

デフォルトでは Feature Report を使い、以下の形で通信します。実際のデバイス仕様に合わせてオプションを調整してください。

- ステータス取得: `[report_id, command_status, 0, 0, ...]` を送信し、応答の `status-index` バイト目が0以外なら点灯と判定
- 点灯/消灯: `[report_id, command_set, on_value/off_value, 0, ...]` を送信

必要に応じて `report-len` や各コマンド値、状態を読む位置を変更してください。

## テスト

```bash
cargo test
```
