// IQC 自動判定サービス（FR-IQ-005/006）
// AQL 規格に基づく入荷検査の自動判定を実装する。
// Accept / Concession / Screening / Reject の 4 区分を判定する。

use crate::model::incoming_inspection::{
    IncomingInspection, IncomingInspectionMeasurement, IqcResult,
};

/// IQC 自動判定サービス。
/// AQL 規格（FR-IQ-005/006）に基づき入荷検査の合否を自動判定する。
pub struct IqcDecisionService;

impl IqcDecisionService {
    /// 新しい IQC 判定サービスを作成する。
    pub fn new() -> Self {
        Self
    }

    /// AQL 自動判定（FR-IQ-005/006）。
    /// 測定値の合否率に基づき 4 区分の判定を行う。
    ///
    /// # 判定基準
    /// - 全測定値合格（100%）: Accept
    /// - 合格率 80% 以上: Concession（特採・偏差承認が必要）
    /// - 合格率 50% 以上: Screening（選別使用）
    /// - 合格率 50% 未満: Reject（不合格）
    pub fn decide(
        _inspection: &IncomingInspection,
        measurements: &[IncomingInspectionMeasurement],
    ) -> IqcResult {
        // 測定値がない場合は判定保留（Concession として扱う）
        if measurements.is_empty() {
            tracing::warn!("IQC 測定値が存在しないため Concession と判定します");
            return IqcResult::Concession;
        }

        let total = measurements.len();
        let pass_count = measurements.iter().filter(|m| m.is_pass).count();

        // 合格率を計算する
        let pass_rate = pass_count as f64 / total as f64;

        // AQL 判定基準に基づき結果を返す（FR-IQ-005/006）
        if (pass_rate - 1.0_f64).abs() < f64::EPSILON {
            // 全数合格: Accept
            IqcResult::Accept
        } else if pass_rate >= 0.8 {
            // 80% 以上合格: Concession（特採・偏差承認が必要）
            IqcResult::Concession
        } else if pass_rate >= 0.5 {
            // 50% 以上合格: Screening（選別使用）
            IqcResult::Screening
        } else {
            // 50% 未満: Reject（不合格）
            IqcResult::Reject
        }
    }
}

impl Default for IqcDecisionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::incoming_inspection::{IqcResult, IqcStatus};
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用の IncomingInspection を作成するヘルパー
    fn make_inspection() -> IncomingInspection {
        IncomingInspection {
            qc_case_id: Uuid::now_v7(),
            lot_id: Uuid::now_v7(),
            sop_id: Uuid::now_v7(),
            status: IqcStatus::Completed,
            inspector_id: Uuid::now_v7(),
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            result: None,
            prev_hash: "0".repeat(64),
            content_hash: "a".repeat(64),
            chain_hash: "b".repeat(64),
        }
    }

    // テスト用の測定値を作成するヘルパー
    fn make_measurement(qc_case_id: Uuid, is_pass: bool) -> IncomingInspectionMeasurement {
        IncomingInspectionMeasurement {
            measurement_id: Uuid::now_v7(),
            qc_case_id,
            step_id: Uuid::now_v7(),
            value: if is_pass { 5.0 } else { 15.0 },
            lower_limit: Some(0.0),
            upper_limit: Some(10.0),
            is_pass,
        }
    }

    #[test]
    fn test_decide_all_pass_returns_accept() {
        // 全数合格の場合 Accept を返すことを確認する（FR-IQ-005）
        let inspection = make_inspection();
        let measurements = vec![
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, true),
        ];
        assert_eq!(
            IqcDecisionService::decide(&inspection, &measurements),
            IqcResult::Accept
        );
    }

    #[test]
    fn test_decide_80_percent_pass_returns_concession() {
        // 80% 合格の場合 Concession を返すことを確認する（FR-IQ-006）
        let inspection = make_inspection();
        let measurements = vec![
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, false),
        ];
        assert_eq!(
            IqcDecisionService::decide(&inspection, &measurements),
            IqcResult::Concession
        );
    }

    #[test]
    fn test_decide_50_percent_pass_returns_screening() {
        // 50% 合格の場合 Screening を返すことを確認する
        let inspection = make_inspection();
        let measurements = vec![
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, false),
        ];
        assert_eq!(
            IqcDecisionService::decide(&inspection, &measurements),
            IqcResult::Screening
        );
    }

    #[test]
    fn test_decide_less_than_50_percent_returns_reject() {
        // 50% 未満合格の場合 Reject を返すことを確認する
        let inspection = make_inspection();
        let measurements = vec![
            make_measurement(inspection.qc_case_id, true),
            make_measurement(inspection.qc_case_id, false),
            make_measurement(inspection.qc_case_id, false),
            make_measurement(inspection.qc_case_id, false),
        ];
        assert_eq!(
            IqcDecisionService::decide(&inspection, &measurements),
            IqcResult::Reject
        );
    }
}
