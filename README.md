# cap-locator-cli

USB HID経由で「locator」を制御するRust製コマンドラインツールです。Cap Locator のPIC16F1455向けHIDファームウェア(Endpoint1 IN/OUT 64B、Feature Report 64B)と通信できるようデフォルト値を合わせています。

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
cargo run -- status --report-id 0 --command-status 1 --status-index 2 --report-len 64
```

`--id <locator-id>` を付けると特定のlocatorだけ確認します。`locator-id` はシリアル番号があればそれを、なければHIDパスを参照します（部分一致も可）。

### LED点灯/消灯

```bash
# 点灯 (RC4/RC5のLEDを点灯させるマスク0x0cがデフォルト)
cargo run -- on

# 消灯
cargo run -- off
```

`--id` を省略した場合は、フィルタに合致するデバイスが1台ならそれを対象にします（複数見つかったらエラーになるので `--id` で指定してください）。`--on-value` / `--off-value` でデバイスが期待する値を指定できます（デフォルトは0x0c/0x00）。

## オプション早見表

- `--vendor-id`, `--product-id` : ベンダー/プロダクトでフィルタ (Cap Locatorは 0x04d8 / 0x1455)
- `--usage-page`, `--usage` : HID Usageでフィルタ (Cap Locatorは 0xFF00 / 0x0001)
- `--report-id` : Feature Reportに使うReport ID（デフォルト0）
- `--report-len` : Feature Report長（デフォルト64バイト）
- `--command-status` : ステータス問い合わせに使うコマンド値（デフォルト0x01）
- `--command-set` : LED制御に使うコマンド値（デフォルト0x02）
- `--status-index` : 応答中どのバイトをLED状態として読むか（デフォルト2。LEDマスクを読み取ります）
- `--on-value` / `--off-value` : 点灯/消灯指示で送るLEDマスク (デフォルト0x0c / 0x00)

## プロトコルについて

Cap Locator HIDファームウェアのFeature Reportは64バイト（Report ID 0）。LEDマスク(bit0〜bit4)を扱い、RC4/RC5が有効なLED出力です。

- ステータス取得: `[0x00, 0x01, 0x00, ...]` を SET_REPORT、続けて GET_REPORT で `[0x00, 0x01, led_mask, ...]` が返ります。`status-index`(=2)の値が0なら消灯、非0なら点灯とみなします。
- 点灯/消灯: `[0x00, 0x02, led_mask, ...]` を SET_REPORT で送信します。デフォルトの `led_mask=0x0c` でRC4/RC5を点灯、`0x00`で全消灯。複数ビットを立てれば対応するLEDを同時に制御できます。

※ ファームウェアの割り込み転送(IN/OUT)は簡易エコー(先頭0xAC)ですが、本CLIはLED制御用のFeature Reportにフォーカスしています。

## テスト

```bash
cargo test
```
