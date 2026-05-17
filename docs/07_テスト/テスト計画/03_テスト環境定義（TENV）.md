# 03 テスト環境定義（TENV）

本章は `04_概要設計/10_テスト方式設計/02_自動化基盤の方式設計` で採番した TENV-001〜008 の各環境について、起動コマンド・DB 設定・破棄手順・分離要件・ネットワーク要件を実行計画として確定する。

---

## 1. 環境一覧

| TENV-ID | 環境名 | Docker Compose プロファイル | 主要ツール |
|---|---|---|---|
| TENV-001 | ユニットテスト環境 | 不要（外部依存なし）| cargo test / Jest |
| TENV-002 | 統合テスト環境 | `test` | PostgreSQL 16-alpine + sqlx::test |
| TENV-003 | E2E Web 環境 | `e2e` | Playwright + axum dev server |
| TENV-004 | E2E RN 環境 | `e2e` + エミュレータ | Detox + Android Emulator API 33 / iOS Simulator 17 |
| TENV-005 | 契約テスト環境 | `e2e` | Schemathesis |
| TENV-006 | 性能テスト環境 | `perf` | k6 + 本番相当スペック + 1 万件シード |
| TENV-007 | セキュリティテスト環境 | `security` | OWASP ZAP |
| TENV-008 | ステージング / UAT 環境 | 本番同等構成 | IIS / Docker（本番スペック）|

**本節で確定した方針**
- **TENV-001〜008 の各環境識別子・プロファイル・主要ツールを本章で確定し、変更時は ADR-TEST-NNN を起票することを確定する。**
- **TENV-002 の DB（wnavdb_test, port 5433, tmpfs）を本番 DB と物理分離することを確定し、同一インスタンスへの相乗りを禁止する。**

---

## 2. TENV-002 統合テスト環境（詳細）

### 起動コマンド

```bash
docker compose --profile test up -d postgres_test
export DATABASE_URL=postgres://wnav_test:test_secret@localhost:5433/wnavdb_test
sqlx migrate run
```

### テスト DB 仕様

| 項目 | 設定値 |
|---|---|
| イメージ | `postgres:16-alpine` |
| DB 名 | `wnavdb_test` |
| ユーザー | `wnav_test` |
| ポート | `5433`（本番 5432 と分離）|
| ストレージ | `tmpfs`（永続化なし）|
| マイグレーション | `migrations/` 配下（本番と同一）|

### 破棄コマンド

```bash
docker compose --profile test down -v
```

テスト実行後は必ず上記コマンドでコンテナとボリュームを破棄する。次回実行時は必ずクリーンな状態から開始する。

**本節で確定した方針**
- **TENV-002 の起動・DB 仕様・破棄手順を確定する。テスト DB 専用マイグレーションファイルの追加を禁止する。**

---

## 3. TENV-003/004 E2E テスト環境（詳細）

### TENV-003 E2E Web（Playwright）

```bash
docker compose --profile e2e up -d
npx playwright test
docker compose --profile e2e down -v
```

### TENV-004 E2E RN（Detox）

| 設定 | 値 |
|---|---|
| Android Emulator | API 33（Pixel 6 相当）|
| iOS Simulator | iOS 17（iPhone 14 相当）|
| 事前条件 | `e2e` プロファイルの API サーバー起動完了後 |

```bash
# Android
npx detox test --configuration android.emu.debug

# iOS
npx detox test --configuration ios.sim.debug
```

**本節で確定した方針**
- **E2E テストは `e2e` プロファイルの API サーバー起動完了後に実施することを確定する。エミュレータ未起動での Detox 実行を禁止する。**

---

## 4. TENV-006 性能テスト環境・TENV-007 セキュリティテスト環境

### TENV-006 性能テスト

| 項目 | 設定値 |
|---|---|
| ツール | k6 |
| シードデータ | `db/seeds/test/` の 4 ファイル + 1 万件追加シード |
| 同時ユーザー | 最大 50 VU |
| 判定基準 | P95 ≤ 2000ms・5xx エラーレート 0% |

```bash
docker compose --profile perf up -d
k6 run tests/performance/main.js
docker compose --profile perf down -v
```

### TENV-007 セキュリティテスト

```bash
docker compose --profile security up -d
# OWASP ZAP ベーススキャン
docker run -t owasp/zap2docker-stable zap-baseline.py \
  -t http://api:8080 \
  -r reports/zap-baseline-$(date +%Y%m%d).html
docker compose --profile security down -v
```

**本節で確定した方針**
- **TENV-006 の性能テストは 1 万件シードを前提とし、シード未適用状態での実行を禁止する。**
- **TENV-007 の ZAP スキャン結果は `reports/zap-baseline-YYYYMMDD.html` に保存し、git 管理することを確定する。**

---

## 5. TENV-008 ステージング / UAT 環境

| 要件 | 内容 |
|---|---|
| スペック | 本番同等（IIS + Windows Server 2022 または WSL2 + Docker）|
| DB | 本番同等スキーマ・UAT 専用シードデータ |
| TLS | 必須（自己署名証明書可）|
| 本番 DB 接続 | 禁止（物理分離）|
| データリセット | UAT 開始前にシードデータをリセット |

**本節で確定した方針**
- **TENV-008 は本番同等スペック・TLS 必須・本番 DB 物理分離の 3 条件を確定する。TLS なしでの UAT 実施を禁止する。**
- **UAT 開始前のデータリセット実施を確定し、前回 UAT のデータが残存した状態での開始を禁止する。**

---

## 参照業界分析

### 必須
- [`../../../90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)

### 関連
- [`../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

---

## 版数履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | RyuheiKiso | 初版 |
