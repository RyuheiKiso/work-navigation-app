# 06 縮退運用発動手順（LEVEL-1〜3）

最終更新: 2026-05-18 | 管理者: system_admin | 根拠要件: OPS-055 / OPS-060 / OPS-061

---

## 1 概要

図: fig_degradation_3level_state（img/ 配下）を参照

縮退運用は 3 段階（LEVEL-1〜3）で定義する。各レベルは上位レベルを包含する（LEVEL-3 は LEVEL-2 も LEVEL-1 も含む）。

| レベル | 名称 | 発動基準 | 発動方式 |
|---|---|---|---|
| LEVEL-1 | API 縮退（読み取り専用） | API エラー率 ≥ 5%（5 分移動平均） | system_admin 確認後手動発動 |
| LEVEL-2 | ハンディ オフラインモード | ハンディ疎通断 60s 連続 | 自動発動 + system_admin 通知 |
| LEVEL-3 | サーバー完全停止・紙運用 | サーバー完全停止 + 復旧見込み ≥ 30 分 | system_admin 手動 + quality_admin 承認 |

---

## 2 LEVEL-1: API 縮退（読み取り専用）

### 2.1 発動基準

| 指標 | 閾値 | 計算式 |
|---|---|---|
| API エラー率 | ≥ 5%（5 分移動平均） | `rate(api_errors_total[5m]) / rate(api_requests_total[5m]) * 100` |

system_admin が Grafana またはログで上記を確認し、発動を判断する。

### 2.2 発動手順

```bash
# Step 1: Prometheus でエラー率確認
curl -fsS "http://localhost:9090/api/v1/query?query=rate(api_errors_total[5m])/rate(api_requests_total[5m])*100" \
  | jq '.data.result[].value[1]'

# Step 2: 縮退モード config flag をセット（API 設定 / 環境変数）
# .env ファイルの DEGRADATION_LEVEL を更新
echo "DEGRADATION_LEVEL=1" >> {PROJECT_PATH}/.env

# Step 3: API コンテナ再起動（設定反映）
docker compose restart api

# Step 4: 縮退モード確認
curl -fsS http://localhost:8080/health | jq '.degradation_level'
# 期待値: 1
```

### 2.3 LEVEL-1 時の動作

| 操作種別 | LEVEL-1 での動作 |
|---|---|
| 読み取り系（GET）| 正常応答 |
| 書き込み系（POST/PUT）| Outbox に蓄積・即時応答は「受付完了」を返す |
| 削除（DELETE）| 拒否（409 Conflict を返す） |
| 認証 | 正常動作 |

### 2.4 利用者への影響説明

```
通知文書（LEVEL-1 発動時）:
「ただいまシステムメンテナンス中のため、作業記録の書き込みに若干の遅延が
発生しています。作業記録は端末に保存され、後ほど自動で反映されます。
作業指示の参照は通常通り使用できます。」
```

### 2.5 解除条件と手順

解除条件: API エラー率 < 1%（5 分移動平均）+ ハートビート 3 回連続成功 + ハッシュチェーン OK

```bash
# 解除手順
sed -i 's/DEGRADATION_LEVEL=1/DEGRADATION_LEVEL=0/' {PROJECT_PATH}/.env
docker compose restart api
curl -fsS http://localhost:8080/health | jq '.degradation_level'
# 期待値: 0
```

**本節で確定した方針**
- **LEVEL-1 は system_admin が 5 分移動平均のエラー率を確認した後に手動で発動する（自動発動は禁止）。**
- **LEVEL-1 発動中に蓄積された Outbox イベントは解除後に自動再送する（RUN-026 の手動再送は不要）。**
- **LEVEL-1 の発動・解除は必ず INC 記録（または保守記録）に時刻と理由を記録する。**

---

## 3 LEVEL-2: ハンディ オフラインモード

### 3.1 発動基準

| 指標 | 閾値 | 発動方式 |
|---|---|---|
| ハンディ→API 疎通 | 断絶 60 秒連続 | 自動発動（ハンディアプリ内） |

ハンディアプリが 60 秒間 API に接続できない場合、アプリは自動的にオフラインモードに切り替わる。system_admin には Prometheus アラート経由で通知される。

### 3.2 自動発動の動作

| 操作 | LEVEL-2 での動作 |
|---|---|
| 作業指示参照 | SQLite ローカルキャッシュを参照（前回同期時点のデータ） |
| 作業実績記録 | SQLite にローカル保存（サーバー未同期） |
| 検査記録 | SQLite にローカル保存（サーバー未同期） |
| 認証 | SQLite に保存された PIN 認証で継続（JWT 再発行は不可）|

### 3.3 system_admin の確認手順

```bash
# Step 1: ハンディ疎通断の確認
curl -fsS "http://localhost:9090/api/v1/query?query=handy_connectivity_up" | jq '.data.result'

# Step 2: API 側の疎通確認
curl -fsS http://localhost:8080/health
ping -c 3 {WIFI_AP_IP}

# Step 3: 原因切り分け（RUN-019）
docker compose logs api --since 30m | grep -E "sync|offline|connection"
```

### 3.4 利用者への影響説明

```
通知文書（LEVEL-2 発動時）:
「ハンディ端末がオフラインモードで動作中です。作業記録は端末に保存されており、
ネットワーク接続が回復次第、自動で同期されます。
作業指示は最終同期時点のデータを参照してください。
最新のマスタデータ（SOP 更新等）は適用されません。」
```

### 3.5 解除条件と手順

解除条件: ハートビート 3 回連続成功 + ハッシュチェーン OK + 未同期データの同期完了確認

ハンディアプリは疎通回復を自動検知してオンラインモードに戻る。同期完了後に system_admin が SQLite sync ログを確認する。

```bash
# 同期完了確認
curl -fsS "http://localhost:8080/api/v1/ops/sync/status" \
  -H "Authorization: Bearer ${SYSTEM_ADMIN_JWT}" | jq '.pending_sync_count'
# 期待値: 0
```

**本節で確定した方針**
- **LEVEL-2 は自動発動する。発動確認後に system_admin は 5 分以内に 03 章 RUN-003 で通知文書を発行する。**
- **オフライン中に記録されたデータは同期後に BAT-003（ハッシュチェーン検証）で整合性を確認する。**
- **オフライン継続が 30 分を超え復旧見込みがない場合は LEVEL-3 の発動を quality_admin と協議する。**

---

## 4 LEVEL-3: サーバー完全停止・紙運用

### 4.1 発動基準

以下の両方を満たすこと:
1. サーバー（API / DB / IIS）が完全停止している（すべての health check が失敗）
2. system_admin が 30 分以内の復旧見込みなしと判断している

### 4.2 発動手順（承認フロー）

```
STEP 1: system_admin が状況を評価
  - RUN-010〜011 を実施済みで復旧不可と判定
  - 復旧見込み ≥ 30 分

STEP 2: quality_admin へ電話で連絡し承認を得る
  - 連絡内容: 「INC-YYYY-NNN、P1 対応中。LEVEL-3 縮退（紙運用）への移行を要請。
    推定復旧時間: X 時間後（HH:MM 予定）」
  - quality_admin の口頭承認後に INC 記録に「承認者: quality_admin、時刻: HH:MM」を記録

STEP 3: 現場監督に LEVEL-3 発動を通知
  - 紙フォールバック（07 章）の発動を指示
  - 必要な紙フォーム（07 章リスト）の準備を指示

STEP 4: INC 記録を更新
  - 縮退レベル: 3 に更新
  - 発動時刻・承認者・復旧見込み時刻を記録
```

### 4.3 LEVEL-3 時の動作

| 操作 | LEVEL-3 での動作 |
|---|---|
| 作業指示参照 | 紙の SOP・作業標準書を参照 |
| 作業実績記録 | 07 章の紙フォームに手書き記録 |
| 検査記録 | 07 章の紙フォームに手書き記録 |
| 不適合記録 | 07 章の紙フォームに手書き記録 |

### 4.4 利用者への影響説明

```
通知文書（LEVEL-3 発動時）:
「システムが停止中のため、紙による作業記録に切り替えます。
各工程の現場監督が紙フォームを配布します。
すべての記録は紙フォームへの手書きで行ってください。
システム復旧後に遡及入力を行います（07 章手順）。
紙フォームには記録日時・担当者 ID・工程名を必ず記入してください。」
```

### 4.5 解除条件と手順

解除条件: サーバー完全復旧 + health check 全件 OK + ハッシュチェーン OK + quality_admin 承認

```bash
# 解除確認
curl -fsS http://localhost:8080/health | jq .
# 全項目 OK を確認

# 解除後の遡及入力確認（07 章手順で実施）
# 紙フォームの全件を quality_admin が確認後に遡及入力
```

**本節で確定した方針**
- **LEVEL-3 は system_admin 単独では発動できない。quality_admin の承認が必須。**
- **LEVEL-3 発動から 6 時間以内に復旧しない場合は BCP（07 章）を発動させる。**
- **LEVEL-3 解除後の遡及入力は quality_admin が全件を確認した後にのみ実施する。**

---

## 参照業界分析

### 必須
- IPA「システム管理基準」4.2.1.c — 縮退運用の発動・管理要件の根拠
- ISO 22301（事業継続管理）— LEVEL-3 発動基準と紙フォールバックの参考規格

### 関連
- 21 CFR Part 11（FDA 電子記録）— 電子記録停止時の紙記録への切り替え要件
- AWS Well-Architected Framework（Reliability Pillar）— グレースフルデグラデーション設計の参考
