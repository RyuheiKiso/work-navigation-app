// wnav_domain ドメインエラー型定義
// ドメイン層で発生するビジネスルール違反・状態エラーを網羅的に定義する。
// thiserror によりエラーメッセージを宣言的に実装する。

/// ドメイン層の統一エラー型。
/// 各バリアントはビジネスルール（BR-BUS-*）または機能要件（FR-*）に対応する。
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// 指定されたリソースが存在しない
    #[error("リソースが見つかりません")]
    NotFound,

    /// ロックステップ違反（BR-BUS-001）: 現在ステップを飛ばして進もうとした
    #[error("ロックステップ違反: current_step={current_step}, attempted={attempted_step}")]
    StepSequenceViolation {
        current_step: u32,
        attempted_step: u32,
    },

    /// 証拠記録が必須であるにも関わらず添付されていない（BR-BUS-003）
    #[error("証拠記録が必須です")]
    EvidenceRequired,

    /// 電子サインが必須であるにも関わらず署名されていない（BR-BUS-004）
    #[error("電子サインが必須です")]
    SignRequired,

    /// 許可されていない状態遷移を試みた（ALG-003）
    #[error("状態遷移が不正です: current={current}, next={next}")]
    InvalidStateTransition { current: String, next: String },

    /// 作業員のスキルレベルが SOP 要求スキルに不足している（BR-BUS-002）
    #[error("スキルレベル不足: required={required}, actual={actual}")]
    InsufficientSkillLevel { required: u8, actual: u8 },

    /// 未公開の SOP で作業を開始しようとした（BR-BUS-012 前提条件）
    #[error("SOP が公開されていません")]
    SopNotPublished,

    /// 別端末がすでに Case を占有している（マルチデバイス排他原則）
    #[error("ケースが別端末に占有されています")]
    CaseLocked { locked_by_terminal: uuid::Uuid },

    /// 楽観ロック競合（他のセッションが先に更新した）
    #[error("楽観ロック競合")]
    OptimisticLockConflict,

    /// IQC 測定値が仕様範囲外（FR-IQ-005/006）
    #[error("IQC 測定値が仕様範囲外です: value={value}, min={min}, max={max}")]
    IqcMeasurementOutOfRange { value: f64, min: f64, max: f64 },

    /// リワーク承認に 2 名が必要（FR-AU-007 Two-Person Integrity）
    #[error("リワーク承認に2名の承認者が必要です（FR-AU-007）")]
    ReworkRequiresTwoApprovers,

    /// 外部キーの重複（冪等性キー二重登録等）
    #[error("外部キーが重複しています: key={key}")]
    DuplicateExternalKey { key: String },

    /// 内部エラー（予期しないシステム障害）
    #[error("内部エラー: {0}")]
    Internal(String),
}
