//! ドメイン値オブジェクト
//!
//! 対応 §: ロードマップ §3.1.1 §10.6.1 §28
//!
//! 同一性ではなく **値そのもの** で等価性が決まる小さな不変型。
//! TaskId／DeviceId／LamportTimestamp／CompletionCriteria を提供する。

// ドメインエラーを参照する
use crate::error::DomainError;
// std::fmt をローカル import
use core::fmt;

// =====================================================================
// TaskId（Task の同一性、§3.1.1 「識別」）
// =====================================================================

/// 作業（Task）のグローバル一意 ID
///
/// 文字列ベースの不変値オブジェクト。空文字は不正とする（§3.1.1）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(String);

impl TaskId {
    /// 文字列から TaskId を生成する
    ///
    /// # Errors
    /// 空または規定外形式の場合は `DomainError::InvalidIdentifier` を返す。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        // 入力を String に変換する
        let v: String = value.into();
        // 空文字を弾く
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("TaskId が空です"));
        }
        // 1024 文字超を弾く（DB スキーマ整合のため）
        if v.len() > 1024 {
            return Err(DomainError::InvalidIdentifier("TaskId が長すぎます"));
        }
        // 妥当値を構築して返す
        Ok(Self(v))
    }

    /// 内部表現（&str）を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        // 内部 String を借用で返す
        &self.0
    }
}

// 表示用に文字列をそのまま出す
impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 内部値を委譲表示
        f.write_str(&self.0)
    }
}

// =====================================================================
// DeviceId（端末識別、§10.6.1 で UUID v7）
// =====================================================================

/// 端末識別子（端末初回登録時に発行する UUID v7、§10.6.1）
///
/// ドメイン層は UUID 生成ライブラリに依存しないため、
/// 文字列で表現し境界 crate（adapter）でパースを担う。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DeviceId(String);

impl DeviceId {
    /// 文字列から DeviceId を構築する
    ///
    /// # Errors
    /// 空文字／長すぎる文字列は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        // 入力を String に変換する
        let v: String = value.into();
        // 空文字を弾く
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("DeviceId が空です"));
        }
        // 64 文字超を弾く（UUID 文字列＋接頭辞を許容するサイズ）
        if v.len() > 64 {
            return Err(DomainError::InvalidIdentifier("DeviceId が長すぎます"));
        }
        // 構築して返す
        Ok(Self(v))
    }

    /// 内部表現（&str）を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        // 内部 String を借用で返す
        &self.0
    }
}

// =====================================================================
// LamportTimestamp（§10.6.1 INV-08 単調性）
// =====================================================================

/// Lamport タイムスタンプ
///
/// 単調増加する u64。ProduceEvent（§10.6.1）で `prev + 1` の関係を保つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LamportTimestamp(u64);

impl LamportTimestamp {
    /// ゼロ値を返す（端末初期化時）
    #[must_use]
    pub const fn zero() -> Self {
        // 内部値 0 で初期化する
        Self(0)
    }

    /// u64 から構築する
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        // 内部値をそのまま設定する
        Self(value)
    }

    /// 単調インクリメント
    ///
    /// # Errors
    /// `u64::MAX` でオーバーフローする場合は `DomainError::LamportNonMonotonic`。
    pub fn next(self) -> Result<Self, DomainError> {
        // checked_add でオーバーフロー検出
        match self.0.checked_add(1) {
            // 成功時は新しい値で返す
            Some(v) => Ok(Self(v)),
            // 失敗時は単調性違反扱い（INV-08）
            None => Err(DomainError::LamportNonMonotonic),
        }
    }

    /// 内部 u64 値を取得する
    #[must_use]
    pub const fn value(self) -> u64 {
        // 内部値を返す
        self.0
    }
}

// =====================================================================
// CompletionCriteria（§3.1.1 完了条件）
// =====================================================================

/// 完了条件の表現
///
/// バリアントは段階的に拡張する（§22 サイクル）。
/// 現状は最小実装として `Manual`（人手判定）と `Photo`（写真証跡）の 2 種を持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionCriteria {
    /// 人手判定（作業者が完了をマーク）
    Manual,
    /// 写真証跡（メディアが添付されていること）
    Photo,
}

impl CompletionCriteria {
    /// 引数で示された証跡が当該完了条件を満たすか判定する
    #[must_use]
    pub fn is_met(&self, evidence: &Evidence) -> bool {
        // バリアント別に判定ロジックを分岐する
        match self {
            // 人手判定はフラグのみで満たされる
            CompletionCriteria::Manual => evidence.manually_marked,
            // 写真証跡はメディアが添付されていることを要求する
            CompletionCriteria::Photo => evidence.photo_attached,
        }
    }
}

/// 完了条件判定のための証跡
///
/// 段階的に追加可能な struct。境界 crate（adapter）で DB から読み出す。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evidence {
    /// 人手で完了マークされたか
    pub manually_marked: bool,
    /// 写真が添付されたか
    pub photo_attached: bool,
}

// =====================================================================
// 単体テスト（§13.1 単体／§13.2 性質ベース）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象を import
    use super::*;
    // 性質ベーステスト
    use proptest::prelude::*;

    // TaskId: 空文字を弾くこと
    #[test]
    fn task_id_rejects_empty() {
        // 空文字での生成を試みる
        let result = TaskId::new("");
        // エラーが返ることを期待する
        assert!(matches!(result, Err(DomainError::InvalidIdentifier(_))));
    }

    // TaskId: 1024 文字超を弾くこと
    #[test]
    fn task_id_rejects_too_long() {
        // 1025 文字を生成する
        let too_long = "x".repeat(1025);
        // 生成を試みる
        let result = TaskId::new(too_long);
        // エラーが返ることを期待する
        assert!(matches!(result, Err(DomainError::InvalidIdentifier(_))));
    }

    // LamportTimestamp: next が単調増加すること
    #[test]
    fn lamport_next_is_monotonic() {
        // ゼロから開始
        let t0 = LamportTimestamp::zero();
        // インクリメント
        let t1 = t0.next().expect("zero next must succeed");
        // 値が 1 増えていることを確認
        assert!(t1 > t0);
        // u64 値も 1 であること
        assert_eq!(t1.value(), 1);
    }

    // 性質ベース: 任意 u64 から next を取ったときの単調性（オーバーフロー時除く）
    proptest! {
        #[test]
        fn prop_lamport_strictly_increasing(seed in 0u64..u64::MAX - 1) {
            // 任意の値から
            let t = LamportTimestamp::from_u64(seed);
            // next は成功する
            let n = t.next().expect("non-overflow");
            // n > t の関係が成り立つ
            prop_assert!(n > t);
        }
    }

    // CompletionCriteria: Manual は manually_marked のみで満たされる
    #[test]
    fn manual_criteria_met_with_flag() {
        // フラグだけ立てた証跡
        let ev = Evidence { manually_marked: true, ..Evidence::default() };
        // Manual は満たされる
        assert!(CompletionCriteria::Manual.is_met(&ev));
        // Photo は満たされない
        assert!(!CompletionCriteria::Photo.is_met(&ev));
    }

    // TaskId::as_str が値を保つこと（mutation testing 検出強化、§13.4.1）
    #[test]
    fn task_id_as_str_preserves_value() {
        // 構築
        let id = TaskId::new("task-xyz").expect("valid");
        // 値が一致すること（"" や "xyzzy" への置換を検出）
        assert_eq!(id.as_str(), "task-xyz");
    }

    // TaskId::Display が as_str と一致すること
    #[test]
    fn task_id_display_matches_as_str() {
        // 構築
        let id = TaskId::new("display-test").expect("valid");
        // フォーマット結果
        let s = format!("{id}");
        // 一致
        assert_eq!(s, "display-test");
    }

    // DeviceId::as_str が値を保つこと
    #[test]
    fn device_id_as_str_preserves_value() {
        let id = DeviceId::new("device-abc").expect("valid");
        assert_eq!(id.as_str(), "device-abc");
    }

    // DeviceId 境界値（64 文字 ok / 65 文字 reject）
    #[test]
    fn device_id_boundary_64_ok_65_reject() {
        // 64 文字は OK
        let ok = "a".repeat(64);
        assert!(DeviceId::new(ok).is_ok());
        // 65 文字は NG
        let ng = "a".repeat(65);
        assert!(DeviceId::new(ng).is_err());
    }

    // TaskId 境界値（1024 文字 ok / 1025 文字 reject）
    #[test]
    fn task_id_boundary_1024_ok_1025_reject() {
        // 1024 文字は OK
        let ok = "x".repeat(1024);
        assert!(TaskId::new(ok).is_ok());
        // 1025 文字は NG
        let ng = "x".repeat(1025);
        assert!(TaskId::new(ng).is_err());
    }

    // LamportTimestamp::value が値を保つ
    #[test]
    fn lamport_value_preserves() {
        let t = LamportTimestamp::from_u64(42);
        assert_eq!(t.value(), 42);
    }
}
