# システム部門向け SDK チュートリアル（日本語）

> 対応 §: ロードマップ §25.1 §3.2.1.4 §17 §17.7 §19.4.2
> 想定所要: 60 分（「30 分で書ける Hello Step」を含む）

## 学習目標

1. アドオン SDK（Rust／AssemblyScript）の構造を理解
2. capability ベース権限（§17.4）の最小権限設計
3. Hello Step／Slack Notify／OPC UA Bridge 3 サンプルを動かす
4. Wasmtime ホスト（§17.5）に組み込む手順

## ハンズオン手順

### 1. 環境セットアップ（10 分）

```bash
# Rust toolchain
rustup target add wasm32-wasi

# ワークスペースを取得
git clone https://github.com/RyuheiKiso/work-navigation-app
cd work-navigation-app

# 依存関係を解決
cargo check --workspace
```

### 2. Hello Step を読む（10 分）

`addon-sdk/examples/hello-step/src/lib.rs` を開く。

- `wna_addon_sdk::Host` trait に依存
- `on_step_completed(host, ctx)` で `host.get_current_task()` → `host.log()` の最小経路
- 必要 capability: `task.read` のみ

### 3. Slack Notify を改造（15 分）

`addon-sdk/examples/slack-notify/` を複製し、Microsoft Teams 通知に改造する課題:

1. `Capability::Notify(NotificationChannel::Chat)` を引き続き使う
2. `host.get_config("teams.webhook.url")` でエンドポイント取得
3. `host.notify(NotificationChannel::Chat, message)` で送信

### 4. OPC UA Bridge を試す（15 分）

`addon-sdk/examples/opc-ua-bridge/src/lib.rs` を開き、`append_tag_samples()` の流れを確認。

- 必要 capability: `task.read`、`task.write`、`net.outbound:opc-ua-server.local`
- payload: JSON 文字列（`source/tag/value`）

### 5. Wasmtime ホストへの組込（10 分）

```rust
use wna_addon_runtime::{ResourceLimits, WasmtimeAddonHost};

#[cfg(feature = "wasmtime-runtime")]
fn run_addon() -> anyhow::Result<()> {
    let host = WasmtimeAddonHost::new(ResourceLimits::SERVER_DEFAULTS)?;
    let wasm_bytes = std::fs::read("./my-addon.wasm")?;
    let granted = vec![Capability::TaskRead, Capability::TaskWrite];
    let required = vec![Capability::TaskRead];
    let r = host.instantiate_and_invoke_start(&wasm_bytes, &granted, &required)?;
    println!("addon returned {r}");
    Ok(())
}
```

`feature = "wasmtime-runtime"` を有効化してビルド:

```bash
cargo build -p wna-addon-runtime --features wasmtime-runtime
```

## 受入観点（§17.7 §19.4.2）

- 30 分以内に Hello Step が動作（§19.4.2 「30 分で書ける Hello Step」）
- 3 サンプル全部が動作する
- 悪意あるアドオンがコア・他アドオン・端末データへ越境できないことを STRIDE Elevation テストで確認

## ライセンス

CC BY 4.0
