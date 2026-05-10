# デモ環境セットアップ

> 対応 §: ロードマップ §14.2 §10.5 §10.2.1
>
> 顧客／社内デモ向けに、`make demo` 一発で「ログイン → 進行中作業の一覧 → 作業ステップ表示」までシナリオが繋がる状態にするためのドキュメント。

## 必要なもの

- Docker / Docker Compose
- Rust toolchain（バックエンドの初回ビルドに使用）
- Node.js + pnpm（フロントエンド dev 起動に使用）

`make doctor` で揃っているか自己診断できる（§14.2）。

## ワンコマンドで起動

```bash
make demo
```

内部では順に以下を実行する：

1. `docker compose up -d` — PostgreSQL とバックエンドを起動
2. `scripts/wait-backend-healthy.sh` — `/readyz` が 200 を返すまで最大 60 秒待機（DB への `SELECT 1` 疎通も含む）
3. `docker compose exec backend wna-backend seed --preset showcase` — シード投入（Rust CLI 経由で adapter 層の upsert を呼ぶ）
4. `scripts/qr-pair.sh terminal-001` — 端末ペアリング QR を表示（未生成ならスキップ）

## 投入されるデータ

すべて upsert ベース。`make demo` を何度実行しても最終状態は同じ。

### ユーザ（credentials）

| user_id | display_name | パスワード |
| --- | --- | --- |
| alice   | Alice Operator   | hello-world |
| bob     | Bob 班長         | hello-world |
| charlie | Charlie 生産技術 | hello-world |

### マスタ

| 種別 | 件数 | 例 |
| --- | --- | --- |
| products   | 3 | `P-A001` ベアリングユニット A1, `P-B100` センサーモジュール B100 |
| equipments | 3 | `EQ-LINE-1` 組立ライン 1, `EQ-INSP-1` 検査台 1 |
| parts      | 5 | `PT-BRG-001` 玉軸受 6204, `PT-BLT-M6` M6 ボルト 等 |

### フロー

| flow_id | バージョン | ステータス | 内容 |
| --- | --- | --- | --- |
| `FL-ASSY-A` | v1 | production | 組立 → 検査 → 梱包（最小 ReactFlow JSON） |

### タスク

| task_id | 状態 | 担当 | current step |
| --- | --- | --- | --- |
| `T-DEMO-001` | Running   | alice   | step02（外観検査） |
| `T-DEMO-002` | Ready     | bob     | — |
| `T-DEMO-003` | Completed | alice   | step03（梱包） |
| `T-DEMO-004` | Idle      | charlie | — |

各タスクには 3 ステップ（部品組立 / 外観検査 / 梱包）が紐付く。

## フロントエンドの起動

```bash
# 端末アプリ（製造員向け、既定で alice/hello-world が埋まる）
pnpm -F terminal dev

# 設定 UI（班長／生産技術向け、既定で charlie/hello-world が埋まる）
pnpm -F config-ui dev
```

開発時のみ `VITE_DEMO_MODE=true` の `.env.development` が読み込まれ、ログイン既定値とデモ警告バナーが表示される。本番ビルド (`pnpm build`) では Vite が `.env.development` を読まないため、デモ既定値が誤って出荷される事故が構造的に起きない。

## やり直したいとき

```bash
make demo-down   # docker compose down -v（DB ボリュームも削除）
make demo        # クリーンな状態から再投入
```

## CLI 直接実行（compose を使わない場合）

ローカル PostgreSQL とバックエンドが既に起動している場合：

```bash
DATABASE_URL=postgres://wna:wna_dev_password@localhost:5432/wna \
  cargo run -p wna-infrastructure -- seed --preset showcase
```

プリセット：

- `--preset minimal` — credentials のみ（開発時の手元動作確認）
- `--preset showcase`（既定） — credentials + マスタ + フロー + タスク + ステップ

## 設計上のポイント

- **adapter 層を貫通**: seed は SQL を書かず、`PostgresCredentialRepository::upsert_credential` / `PostgresMasterRepository::upsert_*` / `PostgresRepository::save` を呼ぶ。ドメイン不変条件（`Task::rehydrate` の精度を含む）が常に同じ径路で守られる。
- **冪等性**: 全テーブル `ON CONFLICT DO UPDATE` で実装済み（既存）。CLI seed の構造的特徴ではなく、adapter の API レベルで保証されている。
- **本番事故の防止**: `wna-backend seed` は明示的に呼び出さない限り走らない。`docker compose up` だけでは seed されない（`make demo` 経由のみ）。
