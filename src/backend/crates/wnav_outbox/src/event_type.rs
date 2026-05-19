// Outbox イベント種別列挙型（MOD-BE-006 §6）
// DB 側 TBL-003 outbox_events.event_type CHECK 制約（DDL-003）と常に同値を維持すること。
// DDL-003 を変更した場合は必ず本 enum も同時更新する（指摘1対応）。

/// Outbox イベント種別。
///
/// DB 側 TBL-003 の `ck_outbox_events_event_type` CHECK 制約と同値を保つ必須規約。
/// CI の `cargo sqlx prepare --check` でコンパイル時整合性を確認する。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxEventType {
    // 作業実績系（既存）
    /// 作業イベント（ステップ完了・作業開始等）
    WorkEvent,
    /// 電子サイン記録
    ElectronicSign,
    /// 証拠ファイル（写真・動画）
    EvidenceFile,
    /// 測定値記録
    Measurement,
    /// 作業中断
    Suspension,
    /// アンドン発報
    AndonAlert,
    /// 不適合記録
    Nonconformity,
    /// CAPA（是正・予防措置）
    Capa,
    /// 改善提案
    KaizenProposal,
    // IQC/リワーク系（DDL-003 拡張・指摘1対応）
    /// 受入検査
    IncomingInspection,
    /// 受入検査測定値
    IncomingInspectionMeasurement,
    /// 特採承認
    ConcessionApproval,
    /// リワーク
    Rework,
    /// 処置決定
    Disposition,
    /// リワーク検証
    ReworkVerification,
    /// リワーク済みロットラベル
    ReworkedLotLabel,
    /// 廃棄記録
    ScrapRecord,
    /// ベンダー返品
    ReturnToVendor,
}

impl OutboxEventType {
    /// DB の CHECK 制約値と一致する文字列を返す（DDL-003 同期必須）。
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::WorkEvent => "work_event",
            Self::ElectronicSign => "electronic_sign",
            Self::EvidenceFile => "evidence_file",
            Self::Measurement => "measurement",
            Self::Suspension => "suspension",
            Self::AndonAlert => "andon_alert",
            Self::Nonconformity => "nonconformity",
            Self::Capa => "capa",
            Self::KaizenProposal => "kaizen_proposal",
            Self::IncomingInspection => "incoming_inspection",
            Self::IncomingInspectionMeasurement => "incoming_inspection_measurement",
            Self::ConcessionApproval => "concession_approval",
            Self::Rework => "rework",
            Self::Disposition => "disposition",
            Self::ReworkVerification => "rework_verification",
            Self::ReworkedLotLabel => "reworked_lot_label",
            Self::ScrapRecord => "scrap_record",
            Self::ReturnToVendor => "return_to_vendor",
        }
    }
}
