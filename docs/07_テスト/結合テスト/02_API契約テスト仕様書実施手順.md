# 02 API契約テスト仕様書実施手順

本章は SLCP-JCF2013 §6.1.1.3「ソフトウェア結合テストプロセス」に対応する L4 API 契約テスト（Schemathesis）の実施手順を確定する。
NFR-QUA-032「OpenAPI 3.1 仕様書から自動生成されるテストケースに対するスキーマ準拠率 100%」および「未定義エラーコード発生率 0 件」の達成を合格条件とする。

---

## 1. 実行環境（TENV-005）

| 項目 | 設定値 |
|---|---|
| 環境識別子 | TENV-005 |
| Schemathesis バージョン | 3.x 系最新安定版 |
| Python バージョン | 3.11 以上 |
| テスト対象ベース URL | `http://localhost:8080` |
| OpenAPI 仕様書パス | `./openapi.yaml`（プロジェクトルート）|
| レポート保存先 | `reports/schemathesis-YYYYMMDD.html` |
| 依存サービス | TENV-002（postgres_test コンテナ）を同時起動した状態で実行する |

---

## 2. 事前準備

### 2-1. Schemathesis インストール

```bash
# pip によるインストール
pip install schemathesis

# バージョン確認
schemathesis --version
```

### 2-2. バックエンドサーバー起動

契約テストは実際の HTTP サーバーに対して実行する。L2 統合テストと同一の TENV-002 上で動作するバックエンドを使用する。

```bash
# TENV-002 コンテナが起動済みであることを確認する
docker compose --profile test ps postgres_test

# テスト用バックエンドサーバーを起動する
export DATABASE_URL="postgres://wnav:wnav@localhost:5433/wnavdb_test"
export JWT_SECRET="test_secret_do_not_use_in_production"
cargo run --bin wnav-server -- --port 8080 &
BACKEND_PID=$!

# サーバー起動待機（ヘルスチェックが 200 を返すまで待機する）
until curl -sf http://localhost:8080/health > /dev/null; do sleep 1; done
echo "Backend server is ready"
```

### 2-3. OpenAPI 仕様書の確認

```bash
# OpenAPI 仕様書の構文検証
python -c "import yaml; yaml.safe_load(open('openapi.yaml'))" && echo "YAML OK"

# バリデーション（openapi-spec-validator 使用）
openapi-spec-validator openapi.yaml
```

---

## 3. Schemathesis 実行コマンド

### 3-1. 標準実行

```bash
# 全エンドポイントの契約テストを実行する
schemathesis run ./openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all \
  --report reports/schemathesis-$(date +%Y%m%d).html \
  --workers 1
```

`--workers 1` を指定する理由: テストデータの競合を防ぎ、TENV-002 への同時接続数を制御するため。

### 3-2. 認証が必要なエンドポイントの実行

認証付きエンドポイントには有効な JWT を付与する。

```bash
# JWT を取得する
TEST_JWT=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"login_id":"OP001","password":"correct_password_hash"}' \
  | jq -r '.access_token')

# 認証付きで全エンドポイントを実行する
schemathesis run ./openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all \
  --header "Authorization: Bearer $TEST_JWT" \
  --report reports/schemathesis-$(date +%Y%m%d)-auth.html \
  --workers 1
```

### 3-3. 除外エンドポイントの指定

```bash
# ファイルアップロードエンドポイントは multipart 送信のため別途手動テストとする（TST-intg-011〜013 で対応済み）
schemathesis run ./openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all \
  --exclude-path "/api/v1/evidences" \
  --header "Authorization: Bearer $TEST_JWT" \
  --report reports/schemathesis-$(date +%Y%m%d)-noevidence.html
```

---

## 4. 合格基準

| 項目 | 基準値 | 根拠 |
|---|---|---|
| スキーマ準拠率 | 100%（NFR-QUA-032）| レスポンスの全フィールドが OpenAPI 3.1 定義スキーマに適合する |
| 未定義エラーコード | 0 件（NFR-QUA-032）| `error` フィールドが openapi.yaml に定義された ERR コードのみを返す |
| 5xx レスポンス | 0 件 | 未ハンドルな内部エラーが発生しないこと |
| テスト実行完了率 | 100% | タイムアウト・接続エラーなしで全ケース完了すること |

FAIL と判定するケース:
- レスポンスボディに openapi.yaml で未定義のフィールドが存在する
- レスポンスボディに openapi.yaml で未定義の `error` コードが存在する
- HTTP 500 以上のレスポンスが 1 件でも発生する
- Schemathesis が `FAILED` ステータスで終了する

---

## 5. レポート保存と確認

```bash
# レポートをブラウザで開く（WSL 環境）
explorer.exe $(wslpath -w reports/schemathesis-$(date +%Y%m%d).html)

# レポートの概要を確認する
schemathesis run ./openapi.yaml --base-url http://localhost:8080 2>&1 | tail -20
```

レポートは `reports/schemathesis-YYYYMMDD.html` 形式で保存し、結合テスト実施結果（`06_結合テスト実施結果テンプレート.md`）にパスを記録する。

---

## 6. テスト後クリーンアップ

```bash
# バックエンドサーバーを停止する
kill $BACKEND_PID

# TENV-002 コンテナを停止する（L2 統合テストと共用している場合は継続）
# 全テスト完了後に実行する
docker compose --profile test down -v
```

---

## 7. 参照先

- テストケース定義: `../../../05_詳細設計/08_テストケース詳細設計/`（契約テストセクション）
- テスト環境定義: `../../../04_概要設計/10_テスト方式設計/02_テスト環境定義.md`（TENV-005）
- 品質ゲート: `../../../04_概要設計/10_テスト方式設計/04_品質ゲートとカバレッジ目標.md`（RGATE-004）
- OpenAPI 仕様書: `openapi.yaml`（プロジェクトルート）

**本節で確定した方針**
- **Schemathesis による契約テストは TENV-005 環境で実施し、`./openapi.yaml` を仕様書パスとして指定することを確定する。**
- **スキーマ準拠率 100%・未定義エラーコード 0 件・5xx レスポンス 0 件をすべて満たす場合に PASS と判定する。1 項目でも未達の場合はリリースゲート RGATE-004 を通過させない。**
- **レポートは `reports/schemathesis-YYYYMMDD.html` に保存し、結合テスト実施結果テンプレートに添付することを必須とする。**

---

## 参照業界分析

- [`../../../90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)（必須）
- [`../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)（関連）
- [`../../../90_業界分析/11_計測・工程能力と統計的品質工学.md`](../../../90_業界分析/11_計測・工程能力と統計的品質工学.md)（関連）

---

## 版数履歴

| バージョン | 日付 | 著者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | RyuheiKiso | 初版 |
