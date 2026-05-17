# 05 アカウント入退職管理手順（OPS-PROC-005）

本手順書の責務は作業者アカウントの入職・退職に伴うアクセス制御変更を確定することである。上流要件 NFR-OPS-050・NFR-SEC-010/020（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-005 |
| **頻度** | イベント駆動（入退職発生時随時） |
| **想定所要時間** | 入職 P50: 20 分 / P95: 40 分 / 退職 P50: 15 分 / P95: 30 分 |
| **実施権限** | system_admin（必須）/ quality_admin（退職時の PII 匿名化確認） |

上流要件:
- NFR-OPS-050: 退職者のアクセス権を退職確定後 24 時間以内に失効させること
- NFR-SEC-010: RBAC ロール割当・剥奪はすべて `maintenance_log` に記録すること
- NFR-SEC-020: 退職 30 日後に PII 匿名化（UUID 置換）・60 日後に完全匿名化を確認すること

**本節で確定した方針**
- 退職処理は退職確定連絡受領から 24 時間以内に JWT 失効・RBAC 剥奪を完了することを確定する。
- PII 匿名化は 30 日・60 日後のスケジュールタスクとして `maintenance_log` に予約エントリを登録することを確定する。
- 入職処理は初回ログイン前に操作訓練確認を完了することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] 対象作業者の雇用形態・所属ライン・担当 SOP が確定している（人事部門から書面で取得）
- [ ] 入職の場合: 端末（ハンディ）が準備されている
- [ ] 退職の場合: 退職確定日・端末回収予定日が確定している
- [ ] `jti_revocations` テーブル・`worker_roles` テーブルへの書き込み権限がある

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。

---

## 3. 事前準備

- [SQL] 対象作業者の現在の権限状態を確認する
  ```sql
  SELECT w.id, w.login_id, w.display_name, w.status,
         array_agg(wr.role_id) AS roles
  FROM workers w
  LEFT JOIN worker_roles wr ON w.id = wr.worker_id AND wr.revoked_at IS NULL
  WHERE w.login_id = $1  -- 対象作業者の login_id
  GROUP BY w.id, w.login_id, w.display_name, w.status;
  ```

**本節で確定した方針**
- 手順実施前に対象作業者の現在の権限状態を必ず確認することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

---

### 4.1 入職手順

#### 4.1.1 作業者レコード作成

- [SQL]
  ```sql
  BEGIN;

  -- 作業者 INSERT
  INSERT INTO workers (id, login_id, display_name, employee_number, line_id, status, created_at)
  VALUES (
    gen_random_uuid(),
    'worker_XXX',          -- 採番規約に従う
    '氏名（表示用）',
    'EMP-YYYYNNNN',        -- 社員番号
    'LINE-01',             -- 所属ライン ID
    'active',
    NOW()
  )
  RETURNING id;

  -- RBAC ロール割当（作業ロール: operator）
  INSERT INTO worker_roles (worker_id, role_id, assigned_at, assigned_by)
  SELECT id, 'operator', NOW(), 'system_admin'
  FROM workers WHERE login_id = 'worker_XXX';

  COMMIT;
  ```

- [CHECK] `RETURNING id` で生成された UUID を記録する。ロールバックしていないことを確認する。

#### 4.1.2 初期 JWT 発行

- [CMD]
  ```bash
  # JWT 発行スクリプト（内部ツール）
  /opt/wnav/bin/issue-jwt \
    --worker-login worker_XXX \
    --device-id "DEVICE-$(date +%Y%m%d)-01" \
    --expiry 90d \
    | tee /tmp/jwt-issue-$(date +%Y%m%d).log

  # JWT ファイルを安全な場所に保管
  chmod 600 /tmp/jwt-issue-$(date +%Y%m%d).log
  ```

- [CHECK] JWT が生成されており `kid` ヘッダが最新の KEY-001〜004 のいずれかを指していること。

#### 4.1.3 端末へのインストールと操作訓練確認

- [CMD] JWT をハンディ端末に転送する手順（別途「端末セットアップ手順書」参照）
  ```bash
  # QR コード生成（端末への JWT 転送用）
  /opt/wnav/bin/gen-qr --jwt-file /tmp/jwt-issue-$(date +%Y%m%d).log \
    --output /tmp/qr-$(date +%Y%m%d).png
  ```

- [SQL] 操作訓練完了フラグを記録する
  ```sql
  UPDATE workers
  SET education_completed_at = NOW(),
      education_confirmed_by = 'system_admin'
  WHERE login_id = 'worker_XXX';
  ```

- [CHECK] `education_completed_at` が NULL でないことを確認する。

---

### 4.2 退職手順（24 時間以内必須）

#### 4.2.1 JWT 即時失効

- [SQL]
  ```sql
  BEGIN;

  -- 対象作業者のすべての有効 JWT を失効させる
  INSERT INTO jti_revocations (jti, revoked_at, reason, worker_id)
  SELECT jt.jti, NOW(), 'employee_separation', w.id
  FROM jwt_tokens jt
  JOIN workers w ON jt.worker_id = w.id
  WHERE w.login_id = 'worker_XXX'
    AND jt.revoked_at IS NULL
    AND jt.expires_at > NOW();

  COMMIT;
  ```

- [CHECK] 失効した JWT の件数を確認し 0 件の場合は元々発行済み JWT がないことを確認する。

#### 4.2.2 RBAC ロール剥奪

- [SQL]
  ```sql
  BEGIN;

  -- すべてのロールを剥奪
  UPDATE worker_roles
  SET revoked_at = NOW(),
      revoked_by = 'system_admin'
  WHERE worker_id = (SELECT id FROM workers WHERE login_id = 'worker_XXX')
    AND revoked_at IS NULL;

  -- 作業者ステータスを inactive に変更
  UPDATE workers
  SET status = 'inactive',
      deactivated_at = NOW()
  WHERE login_id = 'worker_XXX';

  COMMIT;
  ```

- [CHECK] ロール剥奪後に §3 の確認クエリを再実行し `roles = {}` かつ `status = 'inactive'` であることを確認する。

#### 4.2.3 端末ワイプ記録

- [SQL]
  ```sql
  INSERT INTO device_actions (worker_id, device_id, action, executed_at, executed_by, note)
  SELECT id, 'DEVICE-YYYYMMDD-01', 'wipe_requested', NOW(), 'system_admin', '退職による端末ワイプ'
  FROM workers WHERE login_id = 'worker_XXX';
  ```

- [CHECK] 端末物理回収が完了している場合は `action = 'wipe_completed'` として別途 UPDATE する。

#### 4.2.4 PII 匿名化スケジュール予約（30 日後・60 日後）

- [SQL]
  ```sql
  -- 30 日後の PII 匿名化予約
  INSERT INTO maintenance_schedule (task_type, scheduled_at, target_worker_id, status, created_by)
  VALUES (
    'pii_anonymize_partial',
    NOW() + INTERVAL '30 days',
    (SELECT id FROM workers WHERE login_id = 'worker_XXX'),
    'pending',
    'system_admin'
  );

  -- 60 日後の完全匿名化予約
  INSERT INTO maintenance_schedule (task_type, scheduled_at, target_worker_id, status, created_by)
  VALUES (
    'pii_anonymize_complete',
    NOW() + INTERVAL '60 days',
    (SELECT id FROM workers WHERE login_id = 'worker_XXX'),
    'pending',
    'system_admin'
  );
  ```

#### 4.2.5 30 日後 PII 匿名化実施（スケジュール到来時）

- [SQL]
  ```sql
  BEGIN;

  -- display_name を UUID 文字列に置換（PII 削除）
  UPDATE workers
  SET display_name = 'ANONYMIZED-' || gen_random_uuid()::text,
      employee_number = NULL,
      anonymized_partial_at = NOW()
  WHERE id = $1;  -- 退職者の worker_id

  COMMIT;
  ```

#### 4.2.6 60 日後 完全匿名化実施（スケジュール到来時）

- [SQL]
  ```sql
  BEGIN;

  -- login_id も UUID 化し完全匿名化
  UPDATE workers
  SET login_id = 'ANON-' || gen_random_uuid()::text,
      anonymized_complete_at = NOW()
  WHERE id = $1;  -- 退職者の worker_id

  COMMIT;
  ```

- [CHECK] 完全匿名化後に元の `login_id` で検索しても 0 件であることを確認する。

**本節で確定した方針**
- 退職手順 §4.2.1〜4.2.3 は 24 時間以内に完了することを確定する。
- PII 匿名化予約エントリを退職時点で必ず作成することを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| 入職-1 | `workers` テーブルに `status='active'` で INSERT されている | ☐ |
| 入職-2 | `worker_roles` にロールが割当されている | ☐ |
| 入職-3 | `education_completed_at` が NULL でない | ☐ |
| 退職-1 | 全 JWT が `jti_revocations` に登録されている | ☐ |
| 退職-2 | `worker_roles.revoked_at` が全行 NOT NULL である | ☐ |
| 退職-3 | `workers.status = 'inactive'` である | ☐ |
| 退職-4 | `maintenance_schedule` に 30 日・60 日後の匿名化予約が存在する | ☐ |

**本節で確定した方針**
- 退職手順の CHK は全項目が合格するまで 24 時間以内に完了することを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| JWT 失効 SQL が 0 件更新 | 継続（理由確認） | system_admin | `jwt_tokens` テーブルに対象 worker の有効 JWT が存在しないことを確認 |
| 端末回収が遅延（24h 超） | 継続（記録必須） | system_admin・quality_admin | リモートワイプコマンドを即時発行 |
| 退職者が操作継続を試みている（ログ検出） | 即時打ち切り | system_admin・quality_admin | インシデント手順を起動し全アクセスログを保全 |

**本節で確定した方針**
- 退職者による操作継続が確認された場合はインシデントとして扱うことを確定する。

---

## 7. 終了条件と記録

- [SQL] maintenance_log への INSERT（入職）
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'account_onboarding',
    NOW(),
    'system_admin',
    '{"result": "pass", "login_id": "worker_XXX", "role": "operator"}'
  );
  ```

- [SQL] maintenance_log への INSERT（退職）
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'account_offboarding',
    NOW(),
    'system_admin',
    '{"result": "pass", "login_id": "worker_XXX", "jwt_revoked": true, "roles_revoked": true, "pii_schedule_created": true}'
  );
  ```

**本節で確定した方針**
- `maintenance_log` への記録なしにアカウント管理手順完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

入職後に誤ってロールを付与した場合は `worker_roles.revoked_at = NOW()` で即時剥奪する。
退職処理は不可逆操作（JWT 失効・RBAC 剥奪）のためロールバックはしない。
誤って退職処理を行った場合は再入職と同等の手順（§4.1）を実施する。

**本節で確定した方針**
- 退職処理（JWT 失効・RBAC 剥奪）はロールバックを行わず、誤処理の場合は再入職手順で対応することを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | — |
| **関連 ALERT** | ALERT-002（認証失敗多発：退職者アクセス試行の検出） |
| **関連 ERR** | ERR-AUTH-001〜004 |
| **関連 KEY** | KEY-001〜004（JWT 署名鍵） |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- ISO/IEC 27001:2022 Annex A 5.18（アクセス権）、A 5.19（サプライヤーとの情報セキュリティ）
- NIST SP 800-53 Rev.5 AC-2（アカウント管理）、PS-4（人事異動）
- 個人情報保護法（令和 4 年全面施行）第 22 条（安全管理措置）
- NFR-OPS-050、NFR-SEC-010/020（本プロジェクト要件定義）
