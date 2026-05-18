# 08 Case 端末占有アルゴリズム（ALG-026〜028）

本章は TBL-051 case_locks と BAT-013 を使用した case_id 単位の端末排他占有プロトコルを規定する。
1 case_id に対し同時に 1 端末のみが ACTIVE 占有できる（FR-SY-011 / ADR-009）。

---

## 1. 占有獲得（ALG-026）

**トリガ**: POST /api/v1/work-executions または POST /api/v1/work-executions/{id}/resume

**処理**:

```sql
-- atomic な占有試行（競合時に NOTHING で失敗を表現）
INSERT INTO case_locks (case_id, terminal_id, user_id)
VALUES ($1, $2, $3)
ON CONFLICT (case_id) DO NOTHING
RETURNING *;
```

- RETURNING でレコードが取得できた場合 → 占有成功。`lock_status = 'ACTIVE'`
- RETURNING で空の場合 → 占有失敗。HTTP 409 ERR-BIZ-026 を返す。
  レスポンス body に現在の占有端末 ID・取得時刻を含める。

**監査記録**: 占有成功時に work_events に `activity = 'case_lock_acquired'` イベントを INSERT する。

---

## 2. 占有拒否（ALG-027）

**トリガ**: step-events / evidence 等の書込リクエストをハンドリングする CaseLockMiddleware

**処理**:

```sql
SELECT terminal_id FROM case_locks
WHERE case_id = $1 AND lock_status = 'ACTIVE';
```

- `terminal_id = current_terminal_id` → 通過
- `terminal_id != current_terminal_id` または レコードなし → HTTP 409 ERR-BIZ-026

**監査記録**: 拒否発生時に LOG-011 (case_lock.audit) を出力する（work_events には記録しない。頻度が高いため監査ログのみ）。

---

## 3. 正常解放（ALG-028）

**トリガ**: POST /api/v1/work-executions/{id}/suspend または /complete の成功時

**処理**: 同一 TX で case_lock を削除する（履歴保持より可用性を優先）。

```sql
DELETE FROM case_locks
WHERE case_id = $1 AND terminal_id = $2;
```

**監査記録**: 解放時に work_events に `activity = 'case_lock_released'` イベントを INSERT する。

---

## 4. 異常終了時の自動解放（BAT-013）

端末がクラッシュ等で正常解放できなかった場合、BAT-013 が 60 秒ごとに EXPIRED 化する。

```sql
-- BAT-013: heartbeat_at が 5 分以上古い ACTIVE ロックを EXPIRED 化
UPDATE case_locks
SET lock_status = 'EXPIRED'
WHERE lock_status = 'ACTIVE'
  AND heartbeat_at < NOW() - INTERVAL '5 minutes'
RETURNING case_id, terminal_id, user_id;
```

EXPIRED 化された各レコードに対し `activity = 'case_lock_expired'` イベントを work_events に INSERT する。

---

## 5. ハートビート更新

端末は 60 秒ごとにハートビートを送信する。サーバーは以下を実行する:

```sql
UPDATE case_locks
SET heartbeat_at = NOW()
WHERE case_id = $1 AND terminal_id = $2 AND lock_status = 'ACTIVE';
```

ハートビートのエンドポイントは PUT /api/v1/work-executions/{id}/heartbeat（新規採番: API-work-execs-006）。

---

## 6. オフライン中の挙動

- 端末がオフライン（DISCONNECTED / EMERGENCY_MODE）の場合、ハートビート送信が停止する。
- 5 分後に BAT-013 が自動 EXPIRED 化する（Emergency Mode の閾値 300 秒と一致 / D5）。
- EXPIRED 後、他端末が同一 case_id の占有を取得できる。
- オフライン端末が復旧した場合、POST /resume で再占有を試みる（失敗時は ERR-BIZ-026）。

---

## 7. 新規 activity 種別の追加

TBL-001 work_events.activity の CHECK 制約に以下を追加する（DDL-001 マイグレーション対象）:

```sql
ALTER TABLE work_events
  DROP CONSTRAINT ck_work_events_activity,
  ADD CONSTRAINT ck_work_events_activity CHECK (
    activity IN (
      'work_started',
      'step_completed',
      'step_skipped',
      'step_rejected',
      'work_suspended',
      'work_resumed',
      'work_completed',
      'work_cancelled',
      'correction',
      -- Case ロック関連（2026-05-18 追加）
      'case_lock_acquired',
      'case_lock_released',
      'case_lock_expired'
    )
  );
```

---

## 参照

- FR-SY-011（Case 端末排他占有）
- ERR-BIZ-026（case_lock_held_by_other_terminal / HTTP 409）
- BAT-013（case_lock_reaper）
- TBL-051（case_locks）
- IDX-017/018
- ADR-009（マルチデバイス排他方式採用、Phase 5 で作成）
