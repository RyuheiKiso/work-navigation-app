// Case 端末占有ロックのドメインモデル
// マルチデバイス排他原則（src/CLAUDE.md）に基づき 1 case_id = 1 端末を保証する。
// heartbeat で占有状態を維持し、EXPIRED で自動解放する（BAT-013: 5 分閾値）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Case 端末占有ロックエンティティ（TBL-051 case_locks）。
/// 1 case_id = 1 端末を保証するマルチデバイス排他制御に使用する。
/// heartbeat_at が 5 分以上更新されない場合、バッチジョブが EXPIRED に設定する（BAT-013）。
///
/// # 例外制御テーブル
/// このテーブルは app_event_insert ロールに INSERT/UPDATE/DELETE を許可する
/// 例外制御テーブルである（src/CLAUDE.md Append-only 例外）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseLock {
    /// ロック対象ケース ID
    pub case_id: Uuid,
    /// 占有端末 ID
    pub terminal_id: Uuid,
    /// 占有ユーザー ID
    pub locked_by: Uuid,
    /// 占有開始日時
    pub locked_at: DateTime<Utc>,
    /// 最終ハートビート日時（60 秒ごとに更新）
    pub heartbeat_at: DateTime<Utc>,
    /// ロックステータス
    pub status: LockStatus,
}

/// ロックステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LockStatus {
    /// 占有中（ハートビートが有効）
    Active,
    /// 期限切れ（バッチジョブ BAT-013 が設定）
    Expired,
}
