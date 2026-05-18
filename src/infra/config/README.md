# src/infra/config — WNAV 設定ファイル管理

本ディレクトリは WNAV バックエンド・インフラ・フロントエンド配信設定の **単一情報源（SSoT）** である。  
frontend / backend / infra の全レイヤに横断して適用される非機密の接続先情報・パラメータをここで一元管理する。

---

## ファイル一覧

| ファイル | Git 管理 | 用途 |
|---|---|---|
| `config.base.yml` | コミット | 全環境共通の既定値 |
| `config.local.yml.example` | コミット | local プロファイルのテンプレート |
| `config.local.yml` | **除外** | 各開発者のローカル設定実値 |
| `config.dev.yml.example` | コミット | dev プロファイルのテンプレート |
| `config.dev.yml` | **除外** | dev 環境の設定実値 |
| `config.staging.yml` | コミット | staging 環境（secret_ref のみ・実値なし） |
| `config.prod.yml` | コミット | prod 環境（secret_ref のみ・実値なし） |
| `schema/config.schema.json` | コミット | JSON Schema（バリデーション・TS 型生成） |

---

## セットアップ（初回）

```bash
# 1. local 設定ファイルを作成
cp config.local.yml.example config.local.yml

# 2. config.local.yml を自分の環境に合わせて編集

# 3. 機密を .env に記載（ルートの .env.example を参照）

# 4. プロファイルを環境変数に設定
export WNAV_PROFILE=local
```

---

## プロファイル選択と読込の仕組み

`wnav_terminal_api` / `wnav_master_api` は起動時に以下の順序で設定を読み込む:

```
config.base.yml
  └─ config.{WNAV_PROFILE}.yml  （差分のみ記述・map は深いマージ・配列は完全置換）
       └─ 環境変数 WNAV__SECTION__KEY=value  （動的上書き）
            └─ secret_ref の解決  （env / dpapi / docker_secret / file）
```

`WNAV_PROFILE` が未設定の場合はバイナリが exit code 78 で即座に終了する（fail-fast）。

---

## マージ規則

| 項目 | 規則 |
|---|---|
| map（オブジェクト） | 深いマージ。base にあるキーを override が上書き、新規キーは追加 |
| 配列 | **override 側が完全置換**。base の値は引き継がれない |

配列を完全置換にする理由: base に書いた許可 IP や CORS origin が prod で消せなくなる病理を防ぐ。

---

## secret_ref 文法

機密値は YAML に直接書かず、以下の形式で間接参照する:

```yaml
some_field:
  secret_ref: "<scheme>:<identifier>"
```

| scheme | 解決方法 | 想定環境 |
|---|---|---|
| `env:VAR_NAME` | `std::env::var(VAR_NAME)` | local / dev / CI |
| `dpapi:KEY` | Windows DPAPI `Unprotect` で復号 | prod (Windows Server 2022) |
| `docker_secret:NAME` | `/run/secrets/<NAME>` を読む | Docker (dev / staging / prod) |
| `file:/path/to/file` | ファイル全体の内容を文字列として読む | TLS 証明書 PEM など |

**注意**: `secret_ref` を含むオブジェクトに他のキーを混在させてはならない（起動時エラー）。

---

## 環境変数一覧（機密・メタ）

YAML の `secret_ref:` が参照する環境変数は `.env.example`（リポジトリルート）に記載する。  
各変数の詳細は `docs/06_実装/12_環境変数とシークレット一覧.md` を参照。

| 変数名 | 用途 | prod の格納先 |
|---|---|---|
| `WNAV_PROFILE` | プロファイル選択（必須） | OS 環境変数 |
| `WNAV_CONFIG_DIR` | YAML 配置ディレクトリ上書き（任意） | OS 環境変数 |
| `WNAV_DB_PASSWORD_WRITE` | DB write ロール PW | DPAPI / Docker secret |
| `WNAV_DB_PASSWORD_EVENT_INSERT` | DB event_insert ロール PW | DPAPI / Docker secret |
| `WNAV_DB_PASSWORD_READ` | DB read ロール PW | DPAPI / Docker secret |
| `WNAV_BE_JWT_SECRET` | JWT RS256 秘密鍵 PEM | DPAPI / Docker secret |
| `WNAV_BE_JWT_PUBLIC_KEY` | JWT RS256 公開鍵 PEM | DPAPI / Docker secret |
| `WNAV_BE_WEBHOOK_SECRET` | Webhook HMAC 鍵 | DPAPI / Docker secret |
| `WNAV_BE_BACKUP_NOTIFICATION_URL` | バックアップ通知先 URL | DPAPI / Docker secret |
| `WNAV_INFRA_TLS_CERT` | TLS 証明書パス | OS 環境変数 |
| `WNAV_INFRA_TLS_KEY` | TLS 秘密鍵パス | DPAPI |

---

## Docker での配置

```yaml
# docker-compose.yml の volumes セクション例
services:
  wnav_terminal_api:
    volumes:
      - ./src/infra/config:/etc/wnav/config:ro
    environment:
      - WNAV_PROFILE=dev
      - WNAV_CONFIG_DIR=/etc/wnav/config
```

本番では `/etc/wnav/config/` に `config.base.yml` と `config.prod.yml` を配置し、  
パーミッションを `640`（wnav プロセスユーザーのみ読取可）に設定する。

---

## スキーマバリデーション（CI）

JSON Schema バリデーションを CI で実行する（`schema/config.schema.json` を参照）:

```bash
# jsonschema CLI でバリデーション
pip install jsonschema[format] pyyaml
python3 -c "
import json, yaml, jsonschema, sys
schema = json.load(open('schema/config.schema.json'))
for f in sys.argv[1:]:
    data = yaml.safe_load(open(f))
    jsonschema.validate(data, schema)
    print(f'{f}: OK')
" config.base.yml config.staging.yml config.prod.yml
```

---

## 関連ドキュメント

- `docs/06_実装/12_環境変数とシークレット一覧.md` — 変数台帳・保管場所・機微度
- `docs/06_実装/ADR/ADR-IMPL-001_設定管理を_YAML_figment_secret_refで一元化.md` — 採用理由・代替案との比較
- `docs/05_詳細設計/02_バックエンド詳細設計/10_wnav_config詳細設計.md` — `wnav_config` クレート設計
- `docs/06_実装/10_デプロイ手順.md` — 本番での YAML 配置手順
