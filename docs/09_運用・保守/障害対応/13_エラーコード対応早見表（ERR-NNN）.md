# 13 エラーコード対応早見表（ERR-NNN）

最終更新: 2026-05-18（Unit-14: 2バイナリ分割対応） | 管理者: system_admin

---

## 1 概要

本章は全 22 件のエラーコードに対する「HTTP ステータス・ログ検索クエリ・典型原因 Top3・初動 1 行」を一覧する。詳細な復旧手順は 04〜05 章の対応 RUN を参照する。

**バイナリ別エラー発生元**

バックエンドは 2 バイナリ構成のため、エラーコードの発生元バイナリを特定してからログを確認する。

| バイナリ | ポート | 担当エラーカテゴリ |
|---|---|---|
| terminal-api | 8080 | ERR-AUTH（ハンディ端末認証）・ERR-VAL（作業ログ入力検証）・ERR-BIZ（作業業務ロジック）・ERR-DB（ハンディ向けDB）・ERR-SYS-001/005（OutboxWorker パニック・DLQ）|
| master-api | 8081 | ERR-AUTH（管理コンソール認証）・ERR-BIZ-003/005（SOP 承認・ワークフロー）・ERR-DB-003（ハッシュチェーン破断）・ERR-EXT（外部連携）・ERR-SYS-003/004（帳票・マイグレーション）|

**ログ検索基本コマンド**

```bash
# terminal-api ログからエラーコードを検索
docker compose logs terminal-api --since 1h | grep "ERR-{CODE}"

# master-api ログからエラーコードを検索
docker compose logs master-api --since 1h | grep "ERR-{CODE}"

# 両バイナリの直近 30 分のエラーコード出現頻度（まとめて確認）
docker compose logs terminal-api --since 30m | grep -oE 'ERR-[A-Z]+-[0-9]+' | sort | uniq -c | sort -rn
docker compose logs master-api --since 30m | grep -oE 'ERR-[A-Z]+-[0-9]+' | sort | uniq -c | sort -rn
```

---

## 2 ERR-AUTH（認証・認可）

認証エラーはハンディ端末（terminal-api）と管理コンソール（master-api）の両方で発生する。まず発生元バイナリを特定する。

| ERR コード | HTTP | 発生元バイナリ | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|---|
| ERR-AUTH-001 | 401 | terminal-api / master-api | `grep "ERR-AUTH-001"` | (1) JWT 期限切れ (2) 署名鍵不一致 (3) JWT フォーマット不正 | `docker compose logs terminal-api --since 30m \| grep "ERR-AUTH-001"` でリクエスト元確認 |
| ERR-AUTH-002 | 401 | terminal-api | `grep "ERR-AUTH-002"` | (1) PIN 入力誤り (2) PIN 未設定 (3) キーボード入力文字化け | ユーザーに PIN リセット手順を案内する |
| ERR-AUTH-003 | 423 | terminal-api / master-api | `grep "ERR-AUTH-003"` | (1) ログイン失敗 5 回以上 (2) 管理者によるロック (3) 同期前のロック状態 | `psql -c "SELECT user_id, locked_at FROM users WHERE locked_at IS NOT NULL;"` |
| ERR-AUTH-004 | 403 | master-api | `grep "ERR-AUTH-004"` | (1) ロール設定誤り (2) 権限が付与されていない機能へのアクセス (3) 期限切れ権限 | `psql -c "SELECT user_id, role FROM user_roles WHERE user_id='{ID}';"` |

**認証エラー初動ログ確認**

```bash
# ERR-AUTH 全件の発生頻度（直近 1 時間）- 発生元バイナリ別に確認
docker compose logs terminal-api --since 1h | grep -oE 'ERR-AUTH-00[1-4]' | sort | uniq -c | sort -rn
docker compose logs master-api --since 1h | grep -oE 'ERR-AUTH-00[1-4]' | sort | uniq -c | sort -rn

# 特定ユーザーの認証失敗確認（両バイナリ確認）
docker compose logs terminal-api --since 1h | grep "ERR-AUTH" | grep "{USER_ID}"
docker compose logs master-api --since 1h | grep "ERR-AUTH" | grep "{USER_ID}"
```

**本節で確定した方針**
- **ERR-AUTH-001 多発（30 分で 50 件超）は JWT 秘密鍵の侵害を疑い RUN-027 を実施する。**
- **ERR-AUTH-003 のロック解除は system_admin が DB で直接実施し、INC 記録にロック解除対象数を記録する（個人名禁止）。**
- **ERR-AUTH-004 は権限設定の問題であり緊急対応は不要。P4 として次回メンテナンスで対応する。**

---

## 3 ERR-VAL（バリデーション）

| ERR コード | HTTP | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|
| ERR-VAL-001 | 422 | `grep "ERR-VAL-001"` | (1) 必須項目の未入力 (2) データ型不一致 (3) 文字数超過 | ログのリクエストボディを確認し欠損フィールドを特定する |
| ERR-VAL-002 | 422 | `grep "ERR-VAL-002"` | (1) 日時フォーマット不正 (2) 未来日時の入力（Contemporaneous 違反）(3) タイムゾーン不一致 | ハンディ端末の時刻設定（JST）を確認する |
| ERR-VAL-003 | 422 | `grep "ERR-VAL-003"` | (1) 参照 ID が存在しない (2) 削除済みレコードへの参照 (3) 外部キー制約違反 | `psql -c "SELECT id FROM {TABLE} WHERE id='{REF_ID}';"` |
| ERR-VAL-004 | 422 | `grep "ERR-VAL-004"` | (1) 値が許容範囲外（測定値等）(2) 文字コード不正 (3) 禁止文字列の入力 | ログのリクエストボディで範囲外の値を特定する |

**本節で確定した方針**
- **ERR-VAL は作業記録の継続には影響しない。P4 として対応し、入力側の修正またはバリデーション設定を見直す。**
- **ERR-VAL-002（日時不正）が多発する場合はハンディ端末の NTP 設定を確認する。**
- **ERR-VAL が多発（1 時間で 100 件超）する場合は API 変更またはクライアント側バグを疑い P3 に昇格させる。**

---

## 4 ERR-BIZ（業務ロジック）

| ERR コード | HTTP | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|
| ERR-BIZ-001 | 409 | `grep "ERR-BIZ-001"` | (1) 前工程未完了での次工程開始 (2) 工程順序設定誤り (3) 並行実行による順序破壊 | `psql -c "SELECT step_id, status, completed_at FROM steps WHERE order_no < {CURRENT};"` |
| ERR-BIZ-002 | 422 | `grep "ERR-BIZ-002"` | (1) 証拠ファイル未添付 (2) ファイルサイズ超過 (3) 対応ファイル形式以外 | ログの `evidence_validation` エラーを確認する |
| ERR-BIZ-003 | 409 | `grep "ERR-BIZ-003"` | (1) SOP バージョン不一致 (2) 有効期限切れ SOP (3) 差し替え後のキャッシュ | `psql -c "SELECT id, version, valid_until FROM sops WHERE id='{SOP_ID}';"` |
| ERR-BIZ-004 | 409 | `grep "ERR-BIZ-004"` | (1) 校正期限切れ機器の使用 (2) 機器 ID 入力ミス (3) 校正記録の同期遅延 | `psql -c "SELECT id, calibration_due FROM equipment WHERE id='{EQUIP_ID}';"` |
| ERR-BIZ-005 | 409 | `grep "ERR-BIZ-005"` | (1) 承認者の二次確認待ち (2) 権限不足の承認操作 (3) 承認フロー設定誤り | `psql -c "SELECT id, status, approver_id FROM approvals WHERE record_id='{ID}';"` |

**本節で確定した方針**
- **ERR-BIZ-001（工程順序違反）は作業記録への直接影響あり。P3 として現場監督に確認を依頼する。**
- **ERR-BIZ-004（校正期限切れ）は製品品質に直接影響するため、即座に quality_admin に報告し当該機器の使用を停止する。**
- **ERR-BIZ-003 の SOP 不一致はキャッシュクリア後に再発する場合はマスタデータの更新漏れを確認する。**

---

## 5 ERR-DB（データベース）

| ERR コード | HTTP | 発生元バイナリ | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|---|
| ERR-DB-001 | 503 | terminal-api / master-api | `grep "ERR-DB-001"` | (1) PostgreSQL プロセス停止 (2) 接続数枯渇 (3) ネットワーク断 | `pg_isready -h localhost -p 5432` で疎通確認 |
| ERR-DB-002 | 422 | terminal-api / master-api | `grep "ERR-DB-002"` | (1) 参照先レコードが存在しない (2) 削除済みレコードへの FK 参照 (3) データ不整合 | `psql -c "SELECT constraint_name FROM pg_constraint WHERE contype='f' AND conname='{FK}';"` |
| ERR-DB-003 | 503 | **master-api**（HashChainVerifier） | `grep "ERR-DB-003"` | (1) データ改ざん (2) ビット反転（物理障害）(3) 直接 DB 操作による意図しない変更 | **即座に 08 章（ハッシュチェーン破断対応）へ移行する** |

```bash
# ERR-DB-003 発生時の即時確認（master-api ログを確認）
docker compose logs master-api --since 30m | grep "ERR-DB-003"
curl -fsS "http://localhost:9090/api/v1/query?query=hash_chain_error_total" | jq '.data.result'
# 0 より大きい値 → 08 章へ
```

**本節で確定した方針**
- **ERR-DB-001 は P1/P2 として即時 RUN-011 を実施する。**
- **ERR-DB-003 は master-api（HashChainVerifier）が検出する。確認した瞬間に P1 として 08 章へ移行する。自己判断での修復は禁止する。**
- **ERR-DB-002 が多発する場合はデータ移行・マスタ更新のバグを疑い P2 として調査する。**

---

## 6 ERR-EXT（外部連携）

| ERR コード | HTTP | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|
| ERR-EXT-001 | 503 | `grep "ERR-EXT-001"` | (1) 親機（マスタ機）の停止 (2) ネットワーク断 (3) 親機 API 変更 | `curl -fsS http://{PARENT_HOST}:{PORT}/health` で親機疎通確認 |
| ERR-EXT-002 | 503 | `grep "ERR-EXT-002"` | (1) LDAP サービス停止 (2) 認証情報変更（AD パスワード）(3) LDAP ポート（389/636）ブロック | `ldapsearch -x -H ldap://{LDAP_HOST}:389 -D "{BIND_DN}" -w "{PASS}" -b "{BASE}" "(uid=test)"` |

**本節で確定した方針**
- **ERR-EXT-001/002 は外部システム停止が原因であり、本システム側では縮退対応（06 章 LEVEL-1）で対処する。**
- **ERR-EXT-002（LDAP 断）は既存セッションは継続するが、新規ログインが不可となるため P2 として対応する。**
- **外部システムの復旧は外部担当者への連絡と並行して縮退運用を継続する。**

---

## 7 ERR-SYS（システム）

| ERR コード | HTTP | 発生元バイナリ | ログ検索クエリ | 典型原因 Top3 | 初動 1 行 |
|---|---|---|---|---|---|
| ERR-SYS-001 | 500 | terminal-api / master-api | `grep "ERR-SYS-001"` | (1) Rust パニック（プログラムバグ）(2) OOM（メモリ枯渇）(3) スタックオーバーフロー | `docker compose logs terminal-api --since 5m \| grep -E "panic\|SIGABRT\|killed"` |
| ERR-SYS-003 | 500 | **master-api**（帳票生成） | `grep "ERR-SYS-003"` | (1) 帳票テンプレートファイル欠損 (2) PDF 生成ライブラリエラー (3) 出力先ディスク不足 | `docker compose logs master-api --since 30m \| grep "ERR-SYS-003"` でテンプレートパス確認 |
| ERR-SYS-004 | 500 | master-api | `grep "ERR-SYS-004"` | (1) テンプレートと DB スキーマの不整合 (2) マイグレーション未適用 (3) テンプレートバージョン不一致 | `psql -c "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 5;"` |
| ERR-SYS-005 | 503 | **terminal-api**（OutboxWorker） | `grep "ERR-SYS-005"` | (1) DLQ 件数が上限超過 (2) Outbox Worker 停止 (3) DB への書き込みが停止 | `curl http://localhost:8081/api/v1/ops/outbox/dlq/stats \| jq .dlq_total` → 09 章へ |

```bash
# ERR-SYS-001（パニック）発生時の確認 - 発生元バイナリ別に確認
docker compose logs terminal-api --since 5m | grep -E "panic|thread.*panicked|SIGABRT"
docker compose logs master-api --since 5m | grep -E "panic|thread.*panicked|SIGABRT"

# パニック後の再起動確認
docker compose ps terminal-api master-api
# Status が Restarting の場合は RUN-020 を実施（対象バイナリを指定）
```

**本節で確定した方針**
- **ERR-SYS-001（パニック）は P2 として RUN-020 で即時再起動し、パニックログを証拠保全する。**
- **ERR-SYS-005 は ALERT-001 と連動する。09 章の DLQ 復旧手順を実施する。**
- **ERR-SYS-004 はデプロイ後に多発する傾向があり、マイグレーション未適用を確認し適用する。**

---

## 8 エラーコード全件チェックリスト

| カテゴリ | 件数 | 確認状況 |
|---|---|---|
| ERR-AUTH-001〜004 | 4 件 | — |
| ERR-VAL-001〜004 | 4 件 | — |
| ERR-BIZ-001〜005 | 5 件 | — |
| ERR-DB-001〜003 | 3 件 | — |
| ERR-EXT-001〜002 | 2 件 | — |
| ERR-SYS-001/003〜005 | 4 件 | — |
| **合計** | **22 件** | — |

---

## 参照業界分析

### 必須
- IPA「システム管理基準」4.2.2.b — 既知エラーデータベース管理の要件根拠（本章が KDB に相当）
- RFC 7807（Problem Details for HTTP APIs）— HTTP ステータスコードとエラー応答設計の参考

### 関連
- ITIL v4 Problem Management — 既知エラーの管理・早見表の設計参考
- Google SRE Book Chapter 12 — エラー分類とオンコール対応の実践参考
