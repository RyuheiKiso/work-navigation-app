# 06 JWT 鍵ローテーション確認手順（OPS-PROC-006）

本手順書の責務は BAT-010 による自動 JWT 鍵ローテーション後の状態を確定することである。上流要件 NFR-OPS-051（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-006 |
| **頻度** | 四半期（90 日周期、BAT-010 自動実行の翌営業日） |
| **想定所要時間** | P50: 20 分 / P95: 40 分 |
| **実施権限** | system_admin（必須） |

上流要件:
- NFR-OPS-051: JWT 署名鍵を 90 日周期でローテーションし、ローテーション後の健全性を確認すること
- BAT-010: 90 日周期の自動 JWT 鍵ローテーションバッチ（新鍵生成・旧鍵 grace period 設定・旧鍵廃棄）
- KEY-001〜004: 鍵管理識別子（現在有効な鍵: KEY-001、直前 grace period 中: KEY-002、廃棄済み: KEY-003/004）

**本節で確定した方針**
- BAT-010 による自動ローテーション後の確認手順として本手順書を位置づけることを確定する。
- 新鍵の fingerprint 確認・旧鍵の grace period 確認・完全失効確認を順序通りに実施することを確定する。
- grace period（24 時間）中は新旧鍵の並走状態が正常であることを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] BAT-010 が正常完了している（`job_executions` で `BAT-010` の最新が `SUCCESS`）
- [ ] `/etc/wnav/keys/` ディレクトリに最新鍵ファイルが存在する
- [ ] API が起動しており `/health` が 200 を返す
- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] `key_rotation_log` テーブルへの書き込み権限がある

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。

---

## 3. 事前準備

- [SQL] BAT-010 の完了確認
  ```sql
  SELECT job_id, status, started_at, finished_at, error_detail
  FROM job_executions
  WHERE job_id = 'BAT-010'
  ORDER BY started_at DESC
  LIMIT 3;
  ```

- [CHECK] 最新実行が `SUCCESS` かつ `error_detail IS NULL` であること。
  `started_at` が過去 90 日 ± 1 日以内であること。

**本節で確定した方針**
- BAT-010 が `SUCCESS` でない場合は本手順を開始せず BAT-010 のエラー原因を先に解決することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 ステップ 1: 新鍵の fingerprint 確認

- [CMD]
  ```bash
  # 最新の JWT 署名鍵の fingerprint を確認
  NEW_KEY=$(ls -t /etc/wnav/keys/jwt-signing-*.pem | head -1)
  echo "New key file: ${NEW_KEY}"

  # fingerprint（SHA256 ハッシュ）を取得
  NEW_KEY_FINGERPRINT=$(openssl pkey -in "${NEW_KEY}" -pubout -outform DER 2>/dev/null \
    | openssl dgst -sha256 -hex | awk '{print $2}')
  echo "New key fingerprint: ${NEW_KEY_FINGERPRINT}"
  ```

- [SQL] DB の鍵管理テーブルと照合する
  ```sql
  SELECT key_id, fingerprint, created_at, expires_at, status, kid
  FROM jwt_keys
  WHERE status = 'active'
  ORDER BY created_at DESC
  LIMIT 1;
  ```

- [CHECK] シェルで取得した fingerprint と DB の `fingerprint` が一致すること。
  `kid` ヘッダが KEY-001 の識別子と一致すること。

### 4.2 ステップ 2: 旧鍵の grace period 確認

- [SQL]
  ```sql
  -- grace period 中の鍵（旧鍵）を確認
  SELECT key_id, fingerprint, created_at, expires_at, status, kid,
         grace_period_until,
         (grace_period_until > NOW()) AS in_grace_period
  FROM jwt_keys
  WHERE status = 'grace_period'
  ORDER BY created_at DESC;
  ```

- [CHECK] `in_grace_period = true` の鍵が高々 1 件であること。
  `grace_period_until` が BAT-010 の `finished_at` から 24 時間後であること。

### 4.3 ステップ 3: kid ヘッダによる新旧並走状態の確認

- [CMD]
  ```bash
  # API に接続中のセッションが新旧どちらの鍵で発行された JWT を使用しているか確認
  curl -fsS http://localhost:8080/debug/jwt-stats \
    -H "Authorization: Bearer $(cat /tmp/admin-token.txt)" \
    2>/dev/null | jq '{active_kid: .active_kid, grace_kid: .grace_kid}'
  ```

- [SQL]
  ```sql
  -- 現在発行中の JWT の kid 分布を確認
  SELECT kid, count(*) AS token_count,
         min(issued_at) AS oldest, max(issued_at) AS newest
  FROM jwt_tokens
  WHERE revoked_at IS NULL AND expires_at > NOW()
  GROUP BY kid
  ORDER BY token_count DESC;
  ```

- [CHECK] 新鍵（KEY-001 相当の kid）で発行された JWT が存在すること。
  旧鍵（grace period 中の kid）で発行された JWT も grace period 終了まで有効であること。

### 4.4 ステップ 4: grace period 経過後の旧鍵完全失効確認（grace period 終了後に実施）

grace period（24 時間）が経過した後の翌営業日に実施する。

- [SQL]
  ```sql
  -- grace period が終了した旧鍵が revoked になっているか確認
  SELECT key_id, fingerprint, status, revoked_at, kid
  FROM jwt_keys
  WHERE status = 'revoked'
  ORDER BY revoked_at DESC
  LIMIT 3;
  ```

- [SQL]
  ```sql
  -- 旧鍵（grace period 終了済み）で発行された JWT がすべて失効しているか確認
  SELECT count(*) AS remaining_valid_old_tokens
  FROM jwt_tokens jt
  JOIN jwt_keys jk ON jt.kid = jk.kid
  WHERE jk.status = 'revoked'
    AND jt.revoked_at IS NULL
    AND jt.expires_at > NOW();
  ```

- [CHECK] `remaining_valid_old_tokens = 0` であること。
  0 でない場合は旧鍵で発行されたトークンを強制失効させる（§8 参照）。

### 4.5 ステップ 5: 鍵ローテーション履歴への記録

- [SQL]
  ```sql
  INSERT INTO key_rotation_log (
    old_key_id, new_key_id, rotated_at, confirmed_at,
    confirmed_by, grace_period_until, result
  )
  SELECT
    (SELECT key_id FROM jwt_keys WHERE status = 'revoked' ORDER BY revoked_at DESC LIMIT 1),
    (SELECT key_id FROM jwt_keys WHERE status = 'active' ORDER BY created_at DESC LIMIT 1),
    (SELECT finished_at FROM job_executions WHERE job_id = 'BAT-010' ORDER BY started_at DESC LIMIT 1),
    NOW(),
    'system_admin',
    (SELECT grace_period_until FROM jwt_keys WHERE status = 'revoked' ORDER BY revoked_at DESC LIMIT 1),
    'confirmed'
  ;
  ```

**本節で確定した方針**
- ステップ 1〜3 は BAT-010 完了翌営業日に実施し、ステップ 4〜5 は grace period 終了後に実施することを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| CHK-KEY-1 | 新鍵 fingerprint が DB と一致する | ☐ |
| CHK-KEY-2 | grace period 中の旧鍵が高々 1 件かつ `grace_period_until` が正しい | ☐ |
| CHK-KEY-3 | 新鍵の kid で発行された JWT が存在する | ☐ |
| CHK-KEY-4 | grace period 終了後、旧鍵が `revoked` かつ `remaining_valid_old_tokens = 0` | ☐ |
| CHK-KEY-5 | `key_rotation_log` に今回のローテーション記録が存在する | ☐ |

**本節で確定した方針**
- CHK-KEY-1〜3 は BAT-010 翌営業日、CHK-KEY-4〜5 は grace period 終了翌営業日に確認することを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| BAT-010 が FAILED | 即時打ち切り | system_admin | BAT-010 のエラーログを確認し手動ローテーション実施 |
| fingerprint が DB と不一致 | 即時打ち切り | system_admin | 鍵ファイルの整合性調査・セキュリティインシデント評価 |
| 旧鍵で発行された有効 JWT が残存 | 継続（記録必須） | system_admin | §8 の強制失効手順を実施 |
| grace period 終了後も `status='revoked'` でない | 即時対応 | system_admin | 手動で `UPDATE jwt_keys SET status='revoked'` |

**本節で確定した方針**
- fingerprint 不一致はセキュリティインシデントとして扱い即時打ち切りとすることを確定する。

---

## 7. 終了条件と記録

- [SQL] maintenance_log への INSERT
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'jwt_key_rotation_confirmation',
    NOW(),
    'system_admin',
    '{"result": "pass", "new_kid": "KEY-001-v5", "old_kid": "KEY-001-v4", "grace_period_until": "2026-05-19T09:00:00Z", "bat_010_finished_at": "2026-05-18T03:00:00Z"}'
  );
  ```

**本節で確定した方針**
- `maintenance_log` への記録なしに鍵ローテーション確認完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

**旧鍵で発行された有効 JWT の強制失効（異常時）:**

- [SQL]
  ```sql
  BEGIN;

  -- 旧鍵（revoked）で発行されたすべての有効 JWT を強制失効させる
  INSERT INTO jti_revocations (jti, revoked_at, reason, worker_id)
  SELECT jt.jti, NOW(), 'key_rotation_forced_revocation', jt.worker_id
  FROM jwt_tokens jt
  JOIN jwt_keys jk ON jt.kid = jk.kid
  WHERE jk.status = 'revoked'
    AND jt.revoked_at IS NULL
    AND jt.expires_at > NOW();

  COMMIT;
  ```

**本節で確定した方針**
- 旧鍵での有効 JWT 残存は grace period 内であれば正常であり、grace period 終了後に残存する場合のみ強制失効を実施することを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | BAT-010（JWT 鍵自動ローテーション） |
| **関連 ALERT** | — |
| **関連 ERR** | ERR-AUTH-003（無効な kid による認証失敗） |
| **関連 KEY** | KEY-001〜004（JWT 署名鍵） |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- RFC 7517「JSON Web Key (JWK)」§4（鍵識別子 kid）
- RFC 7519「JSON Web Token (JWT)」§4.1（Claims）
- NIST SP 800-57 Part 1 Rev.5「Recommendation for Key Management」§5.3（鍵ローテーション期間）
- NFR-OPS-051（本プロジェクト要件定義）
