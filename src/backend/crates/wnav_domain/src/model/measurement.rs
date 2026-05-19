// 測定値のドメインモデル
// 数値測定ステップで記録される測定値を管理する。
// 工程能力指数（Cp/Cpk）はバッチ集計で算出する（FR-IQ-005/006）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 測定値エンティティ。
/// 数値測定ステップ（StepType::Measurement）で記録する。
/// 上下限から外れた値は IqcMeasurementOutOfRange エラーを発生させる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// 測定 ID（UUID v7）
    pub measurement_id: Uuid,
    /// 関連する作業実行 ID
    pub work_execution_id: Uuid,
    /// 関連するステップ ID
    pub step_id: Uuid,
    /// 測定値
    pub value: f64,
    /// 単位（例: "mm", "kg", "℃"）
    pub unit: String,
    /// 公称値（設計値）
    pub nominal: Option<f64>,
    /// 上限値（Upper Specification Limit）
    pub upper_limit: Option<f64>,
    /// 下限値（Lower Specification Limit）
    pub lower_limit: Option<f64>,
    /// 工程能力指数 Cp（バッチ集計で算出）
    pub cp: Option<f64>,
    /// 工程能力指数 Cpk（バッチ集計で算出）
    pub cpk: Option<f64>,
}
