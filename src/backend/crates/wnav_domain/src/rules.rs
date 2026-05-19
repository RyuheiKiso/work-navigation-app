// ドメインルール関数（BR-BUS-001〜046）
// ビジネスルールを純粋関数として実装する。
// ドメインサービスから呼び出され、ルール違反時は DomainError を返す。

use uuid::Uuid;

use crate::error::DomainError;
use crate::model::disposition::DispositionApproval;
use crate::model::sop::Sop;
use crate::model::step::Step;
use crate::model::user::User;
use crate::model::work_execution::WorkExecution;

// 個人別労務監視機能の実装禁止（倫理品質・NFR-ETH-001）
// この static はリポジトリ層でも domain 層でも個人を特定した労務監視クエリを禁じる宣言である。
#[allow(dead_code)]
const _INDIVIDUAL_METRICS_FORBIDDEN: () = ();

/// (BR-BUS-001) ロックステップ強制。
/// 現在ステップの番号と試みたステップの番号が一致しているかを検証する。
/// ステップスキップは ERR-BIZ-001 に対応する。
pub fn br_bus_001_lock_step(execution: &WorkExecution, step: &Step) -> Result<(), DomainError> {
    // step_number は 1 基準、current_step_index は 0 基準
    // 次に実行すべきステップは current_step_index + 1
    let expected_step_number = execution.current_step_index + 1;
    if step.step_number != expected_step_number {
        return Err(DomainError::StepSequenceViolation {
            current_step: execution.current_step_index,
            attempted_step: step.step_number,
        });
    }
    Ok(())
}

/// (BR-BUS-002) スキルゲート。
/// ユーザーのスキルレベルが SOP に要求されるスキルレベルを満たしているかを検証する。
/// SOP の required_skill_level はここでは Sop 構造体に持たせず、
/// 呼び出し元が required_skill_level を渡す設計とする。
pub fn br_bus_002_skill_gate(
    user: &User,
    sop: &Sop,
    required_skill_level: u8,
) -> Result<(), DomainError> {
    // SOP が公開されていない場合は作業開始不可
    if !matches!(sop.status, crate::model::sop::SopStatus::Published) {
        return Err(DomainError::SopNotPublished);
    }

    // ユーザーのスキルレベルが要求値に満たない場合はエラー
    if user.skill_level < required_skill_level {
        return Err(DomainError::InsufficientSkillLevel {
            required: required_skill_level,
            actual: user.skill_level,
        });
    }
    Ok(())
}

/// (BR-BUS-003) 証拠記録必須チェック。
/// ステップが証拠必須（evidence_required = true）の場合、
/// evidence_ids が空でないことを検証する（ERR-BIZ-003）。
pub fn br_bus_003_evidence_required(step: &Step, evidence_ids: &[Uuid]) -> Result<(), DomainError> {
    if step.evidence_required && evidence_ids.is_empty() {
        return Err(DomainError::EvidenceRequired);
    }
    Ok(())
}

/// (BR-BUS-004) 電子サイン必須チェック。
/// ステップが電子サイン必須（sign_required = true）の場合、
/// sign_id が Some であることを検証する（ERR-BIZ-002）。
pub fn br_bus_004_sign_required(step: &Step, sign_id: Option<Uuid>) -> Result<(), DomainError> {
    if step.sign_required && sign_id.is_none() {
        return Err(DomainError::SignRequired);
    }
    Ok(())
}

/// (BR-BUS-012) 公開承認に電子サイン必須。
/// MasterVersion を Published に遷移する際、電子サイン ID が必須であることを検証する。
pub fn br_bus_012_publish_requires_sign(sign_id: Option<Uuid>) -> Result<(), DomainError> {
    if sign_id.is_none() {
        return Err(DomainError::SignRequired);
    }
    Ok(())
}

/// (FR-AU-007) Two-Person Integrity チェック。
/// 処置判定（Disposition）には 2 名の独立した承認者が必要であることを検証する。
/// 同一人物による 2 回の承認は不正。
pub fn fr_au_007_two_person_integrity(
    approvals: &[DispositionApproval],
) -> Result<(), DomainError> {
    // 承認者数が 2 未満の場合はエラー
    if approvals.len() < 2 {
        return Err(DomainError::ReworkRequiresTwoApprovers);
    }

    // 第 1 承認者と第 2 承認者が同一人物でないことを確認する
    if approvals.len() >= 2 && approvals[0].approver_id == approvals[1].approver_id {
        return Err(DomainError::ReworkRequiresTwoApprovers);
    }

    Ok(())
}

/// (BR-BUS-041) スキルレベルチェック（スキルゲート拡張版）。
/// br_bus_002_skill_gate の SOP なし版。
/// 単純にユーザーのスキルレベルと要求スキルレベルを比較する。
pub fn br_bus_041_skill_level_check(
    user: &User,
    required_skill_level: u8,
) -> Result<(), DomainError> {
    if user.skill_level < required_skill_level {
        return Err(DomainError::InsufficientSkillLevel {
            required: required_skill_level,
            actual: user.skill_level,
        });
    }
    Ok(())
}

/// (BR-BUS-022) DSL ネスト深度チェック。
/// JSON Logic の condition_dsl のネスト深度が 5 以内であることを検証する。
pub fn br_bus_022_dsl_depth_check(rule: &serde_json::Value) -> Result<(), DomainError> {
    use crate::service::json_logic_evaluator::JsonLogicEvaluator;
    JsonLogicEvaluator::validate_rule_depth(rule, 0, 5)
        .map_err(|e| DomainError::Internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::disposition::DispositionApproval;
    use crate::model::sop::SopStatus;
    use crate::model::step::StepType;
    use crate::model::work_execution::WorkExecutionStatus;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用の WorkExecution を作成するヘルパー
    fn make_execution(current_step_index: u32) -> WorkExecution {
        WorkExecution {
            work_execution_id: Uuid::now_v7(),
            sop_version_id: Uuid::now_v7(),
            primary_worker_id: Uuid::now_v7(),
            secondary_worker_id: None,
            terminal_id: Uuid::now_v7(),
            production_target_id: None,
            status: WorkExecutionStatus::InProgress,
            current_step_index,
            started_at: Some(Utc::now()),
            completed_at: None,
            updated_at: Utc::now(),
        }
    }

    // テスト用の Step を作成するヘルパー
    fn make_step(step_number: u32, evidence_required: bool, sign_required: bool) -> Step {
        Step {
            step_id: Uuid::now_v7(),
            sop_id: Uuid::now_v7(),
            step_number,
            title: "テストステップ".to_string(),
            instruction: "テスト手順".to_string(),
            condition_dsl: None,
            evidence_required,
            sign_required,
            skippable: false,
            estimated_duration_secs: None,
            step_type: StepType::Standard,
        }
    }

    // テスト用の User を作成するヘルパー
    fn make_user(skill_level: u8) -> User {
        User {
            user_id: Uuid::now_v7(),
            login_id: "test_user".to_string(),
            password_hash: "$2b$12$...".to_string(),
            display_name: "テストユーザー".to_string(),
            email: None,
            factory_id: Uuid::now_v7(),
            roles: vec![crate::model::user::RoleId::Operator],
            skill_level,
            is_active: true,
        }
    }

    #[test]
    fn test_br_bus_001_lock_step_ok() {
        // 正常なステップ進行（current=0 → step_number=1）が OK を返すことを確認する
        let execution = make_execution(0);
        let step = make_step(1, false, false);
        assert!(br_bus_001_lock_step(&execution, &step).is_ok());
    }

    #[test]
    fn test_br_bus_001_lock_step_skip_error() {
        // ステップスキップ（current=0 → step_number=2）が StepSequenceViolation を返すことを確認する
        let execution = make_execution(0);
        let step = make_step(2, false, false);
        assert!(matches!(
            br_bus_001_lock_step(&execution, &step),
            Err(DomainError::StepSequenceViolation {
                current_step: 0,
                attempted_step: 2
            })
        ));
    }

    #[test]
    fn test_br_bus_003_evidence_required_ok() {
        // 証拠添付済みで evidence_required=true のステップが OK を返すことを確認する
        let step = make_step(1, true, false);
        let evidence_ids = vec![Uuid::now_v7()];
        assert!(br_bus_003_evidence_required(&step, &evidence_ids).is_ok());
    }

    #[test]
    fn test_br_bus_003_evidence_required_empty_error() {
        // 証拠なしで evidence_required=true のステップが EvidenceRequired を返すことを確認する
        let step = make_step(1, true, false);
        assert!(matches!(
            br_bus_003_evidence_required(&step, &[]),
            Err(DomainError::EvidenceRequired)
        ));
    }

    #[test]
    fn test_br_bus_004_sign_required_ok() {
        // 電子サイン済みで sign_required=true のステップが OK を返すことを確認する
        let step = make_step(1, false, true);
        assert!(br_bus_004_sign_required(&step, Some(Uuid::now_v7())).is_ok());
    }

    #[test]
    fn test_br_bus_004_sign_required_none_error() {
        // 電子サインなしで sign_required=true のステップが SignRequired を返すことを確認する
        let step = make_step(1, false, true);
        assert!(matches!(
            br_bus_004_sign_required(&step, None),
            Err(DomainError::SignRequired)
        ));
    }

    #[test]
    fn test_br_bus_012_publish_requires_sign_ok() {
        // 電子サインあり → OK を返すことを確認する（BR-BUS-012）
        assert!(br_bus_012_publish_requires_sign(Some(Uuid::now_v7())).is_ok());
    }

    #[test]
    fn test_br_bus_012_publish_requires_sign_none_error() {
        // 電子サインなし → SignRequired を返すことを確認する（BR-BUS-012）
        assert!(matches!(
            br_bus_012_publish_requires_sign(None),
            Err(DomainError::SignRequired)
        ));
    }

    #[test]
    fn test_fr_au_007_two_person_integrity_ok() {
        // 2 名の異なる承認者 → OK を返すことを確認する（FR-AU-007）
        let approver1 = Uuid::now_v7();
        let approver2 = Uuid::now_v7();
        let disposition_id = Uuid::now_v7();
        let approvals = vec![
            DispositionApproval {
                approval_id: Uuid::now_v7(),
                disposition_id,
                approver_id: approver1,
                sequence: 1,
                approved_at: Utc::now(),
                comment: None,
            },
            DispositionApproval {
                approval_id: Uuid::now_v7(),
                disposition_id,
                approver_id: approver2,
                sequence: 2,
                approved_at: Utc::now(),
                comment: None,
            },
        ];
        assert!(fr_au_007_two_person_integrity(&approvals).is_ok());
    }

    #[test]
    fn test_fr_au_007_same_person_error() {
        // 同一人物による 2 回承認 → ReworkRequiresTwoApprovers を返すことを確認する（FR-AU-007）
        let approver = Uuid::now_v7();
        let disposition_id = Uuid::now_v7();
        let approvals = vec![
            DispositionApproval {
                approval_id: Uuid::now_v7(),
                disposition_id,
                approver_id: approver,
                sequence: 1,
                approved_at: Utc::now(),
                comment: None,
            },
            DispositionApproval {
                approval_id: Uuid::now_v7(),
                disposition_id,
                approver_id: approver, // 同一人物
                sequence: 2,
                approved_at: Utc::now(),
                comment: None,
            },
        ];
        assert!(matches!(
            fr_au_007_two_person_integrity(&approvals),
            Err(DomainError::ReworkRequiresTwoApprovers)
        ));
    }

    #[test]
    fn test_br_bus_002_skill_gate_published_ok() {
        // 公開済み SOP + 十分なスキルレベル → OK を返すことを確認する（BR-BUS-002）
        let user = make_user(3);
        let sop = Sop {
            sop_id: Uuid::now_v7(),
            operation_id: Uuid::now_v7(),
            name_json: serde_json::json!({"ja": "テスト SOP", "en": "Test SOP"}),
            version: "1.0.0".to_string(),
            status: SopStatus::Published,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(br_bus_002_skill_gate(&user, &sop, 2).is_ok());
    }

    #[test]
    fn test_br_bus_002_skill_gate_insufficient_level() {
        // スキルレベル不足 → InsufficientSkillLevel を返すことを確認する（BR-BUS-002）
        let user = make_user(1);
        let sop = Sop {
            sop_id: Uuid::now_v7(),
            operation_id: Uuid::now_v7(),
            name_json: serde_json::json!({"ja": "テスト SOP"}),
            version: "1.0.0".to_string(),
            status: SopStatus::Published,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(matches!(
            br_bus_002_skill_gate(&user, &sop, 3),
            Err(DomainError::InsufficientSkillLevel {
                required: 3,
                actual: 1
            })
        ));
    }
}
