//! 同期ドメイン
//!
//! 対応 §: ロードマップ §10.6 §10.6.1 §10.6.2 §27 F-002 §29 R-016
//!
//! TLA+ 仕様（[`docs/03_設計/形式化/sync.tla`]）の状態空間に対応する
//! Rust の値型・遷移ロジックを提供する。INV-01〜08 を **実装側でも** 守る。

// 値オブジェクト
use crate::value_object::{DeviceId, LamportTimestamp, TaskId};
// エラー
use crate::error::DomainError;

/// 同期イベント種別（TLA+ EventKind と整合）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncEventKind {
    /// 作業実績（G-Set 追記、§10.6.1）
    Record,
    /// ユーザ設定（LWW-Register 更新）
    UserSetting,
}

/// 同期イベント
///
/// TLA+ の `Event` レコードと等価。`device`／`lamport`／`kind`／`payload` を保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEvent {
    /// 発生端末
    pub device_id: DeviceId,
    /// Lamport timestamp（INV-08 単調性）
    pub lamport: LamportTimestamp,
    /// 種別
    pub kind: SyncEventKind,
    /// 関連 Task ID（record の場合）
    pub task_id: Option<TaskId>,
    /// 設定キー（user_setting の場合）
    pub setting_key: Option<String>,
    /// payload（JSON 文字列）
    pub payload: String,
}

impl SyncEvent {
    /// 作業実績イベントを構築する
    ///
    /// # Errors
    /// Task ID が未指定の場合はドメイン規則違反。
    pub fn record(
        device_id: DeviceId,
        lamport: LamportTimestamp,
        task_id: TaskId,
        payload: impl Into<String>,
    ) -> Self {
        // 値を組み立てる
        Self {
            device_id,
            lamport,
            kind: SyncEventKind::Record,
            task_id: Some(task_id),
            setting_key: None,
            payload: payload.into(),
        }
    }

    /// 設定イベントを構築する
    ///
    /// # Errors
    /// 設定キーが空の場合はドメイン規則違反。
    pub fn user_setting(
        device_id: DeviceId,
        lamport: LamportTimestamp,
        setting_key: impl Into<String>,
        payload: impl Into<String>,
    ) -> Result<Self, DomainError> {
        // 設定キーを取得
        let key: String = setting_key.into();
        // 空文字を弾く
        if key.is_empty() {
            return Err(DomainError::InvalidIdentifier("setting_key が空です"));
        }
        // 値を組み立てる
        Ok(Self {
            device_id,
            lamport,
            kind: SyncEventKind::UserSetting,
            task_id: None,
            setting_key: Some(key),
            payload: payload.into(),
        })
    }
}

/// LWW 比較関数
///
/// `(lamport_ts, device_id)` の lex 順で a が b より「新しい」場合 true を返す（INV-02）。
#[must_use]
pub fn lww_strictly_after(a: (LamportTimestamp, &DeviceId), b: (LamportTimestamp, &DeviceId)) -> bool {
    // Lamport を最優先で比較
    if a.0 > b.0 {
        return true;
    }
    if a.0 < b.0 {
        return false;
    }
    // タイブレーカは device_id の lex 順
    a.1 > b.1
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // record コンストラクタ: kind=Record, task_id 設定
    #[test]
    fn record_event_has_task_id() {
        // ID 値オブジェクト
        let dev = DeviceId::new("d-1").expect("valid");
        let ts = LamportTimestamp::from_u64(7);
        let task = TaskId::new("t-1").expect("valid");
        // 構築
        let ev = SyncEvent::record(dev, ts, task, r#"{"k":"v"}"#);
        // 検査
        assert_eq!(ev.kind, SyncEventKind::Record);
        assert!(ev.task_id.is_some());
        assert!(ev.setting_key.is_none());
    }

    // user_setting: 空キーは拒否
    #[test]
    fn user_setting_rejects_empty_key() {
        let dev = DeviceId::new("d-1").expect("valid");
        let ts = LamportTimestamp::zero();
        let r = SyncEvent::user_setting(dev, ts, "", "{}");
        assert!(matches!(r, Err(DomainError::InvalidIdentifier(_))));
    }

    // LWW 比較: lamport が大きい方が勝つ
    #[test]
    fn lww_lamport_dominates() {
        let d1 = DeviceId::new("d-1").expect("valid");
        let d2 = DeviceId::new("d-2").expect("valid");
        let t1 = LamportTimestamp::from_u64(1);
        let t2 = LamportTimestamp::from_u64(2);
        // (t2, d1) > (t1, d2) は Lamport 比較で true
        assert!(lww_strictly_after((t2, &d1), (t1, &d2)));
        // 逆は false
        assert!(!lww_strictly_after((t1, &d2), (t2, &d1)));
    }

    // LWW 比較: lamport 同値時は device_id lex 順
    #[test]
    fn lww_tiebreak_by_device_id() {
        let d_a = DeviceId::new("d-a").expect("valid");
        let d_b = DeviceId::new("d-b").expect("valid");
        let t = LamportTimestamp::from_u64(5);
        // d-b > d-a
        assert!(lww_strictly_after((t, &d_b), (t, &d_a)));
        // 同 lamport, 同 device_id は false
        assert!(!lww_strictly_after((t, &d_a), (t, &d_a)));
    }
}
