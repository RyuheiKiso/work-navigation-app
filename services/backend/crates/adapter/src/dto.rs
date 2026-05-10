//! API DTO とドメイン型の相互変換
//!
//! 対応 §: ロードマップ §10.3.1 §28
//!
//! プレゼンテーション層／API クライアントが扱う型を本モジュールに集約する。

// シリアライズ／デシリアライズ
use serde::{Deserialize, Serialize};

/// Task の API 表現（GET /tasks/:id 等で返却する DTO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    /// 識別子（§3.1.1 識別）
    pub id: String,
    /// 状態ラベル（HSM の状態名、§3.4.1）
    pub state: String,
    /// 主体端末の ID（§10.6.1）
    pub device_id: String,
    /// 直近の Lamport タイムスタンプ
    pub lamport: u64,
    /// スキーマバージョン（§10.3.1 schema_version）
    pub schema_version: u32,
}

impl TaskDto {
    /// ドメイン Task から DTO を生成する
    #[must_use]
    pub fn from_domain(task: &wna_domain::Task) -> Self {
        // 各フィールドを引き出して文字列／数値に変換する
        Self {
            id: task.id().as_str().to_string(),
            state: task.state().label().to_string(),
            device_id: task.device_id().as_str().to_string(),
            lamport: task.lamport().value(),
            // スキーマ初版のバージョン番号
            schema_version: 1,
        }
    }
}

/// Append Record API リクエスト DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendRecordRequestDto {
    /// 対象 Task の ID
    pub task_id: String,
    /// 発生端末 ID
    pub device_id: String,
    /// Lamport タイムスタンプ
    pub lamport: u64,
    /// 自由形式 payload（後段で JSON にパース可能）
    pub payload: serde_json::Value,
}

// =====================================================================
// 単体テスト（§13.1）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // ドメイン依存
    use wna_domain::{CompletionCriteria, DeviceId, Task, TaskId};

    // from_domain: フィールドが期待通り反映される
    #[test]
    fn task_dto_from_domain_round_trips_fields() {
        // テスト用 ID
        let id = TaskId::new("t-dto").expect("valid id");
        // テスト用 Device
        let dev = DeviceId::new("d-dto").expect("valid id");
        // Task を作る
        let task = Task::create(id.clone(), CompletionCriteria::Manual, dev.clone());
        // DTO に変換
        let dto = TaskDto::from_domain(&task);
        // ID が一致
        assert_eq!(dto.id, id.as_str());
        // 初期状態は Idle
        assert_eq!(dto.state, "Idle");
        // 主体端末 ID
        assert_eq!(dto.device_id, dev.as_str());
        // Lamport は 0
        assert_eq!(dto.lamport, 0);
        // スキーマバージョン
        assert_eq!(dto.schema_version, 1);
    }
}
