// StepEngine サービス（ALG-001〜003）
// CanAdvanceToStep・CompleteStep・WorkExecution ステートマシンを実装する。
// JSON Logic 評価（ALG-004/005）で条件式（condition_dsl）を評価する。

use std::sync::Arc;

use uuid::Uuid;

use crate::error::DomainError;
use crate::model::step::Step;
use crate::repository::StepRepository;
use crate::service::json_logic_evaluator::JsonLogicEvaluator;

/// StepEngine サービス。
/// ALG-001 CanAdvanceToStep・ALG-002 CompleteStep の中核ロジックを実装する。
pub struct StepEngineService {
    /// ステップリポジトリ（ステップ定義の取得に使用）
    pub step_repo: Arc<dyn StepRepository>,
    /// JSON Logic 評価器（condition_dsl の評価に使用）
    pub evaluator: JsonLogicEvaluator,
}

impl StepEngineService {
    /// 新しい StepEngineService を作成する。
    pub fn new(step_repo: Arc<dyn StepRepository>) -> Self {
        Self {
            step_repo,
            evaluator: JsonLogicEvaluator::new(),
        }
    }

    /// ステップを ID で取得する。
    pub async fn get_step(&self, step_id: Uuid) -> Result<Option<Step>, DomainError> {
        self.step_repo.find_by_id(step_id).await
    }

    /// (ALG-002) ステップの完了条件を評価する。
    /// condition_dsl が None の場合は常に true（条件なし）を返す。
    pub fn evaluate_completion_condition(
        step: &Step,
        context: &serde_json::Value,
    ) -> bool {
        // condition_dsl が未設定の場合は常に完了可能（条件なし）
        let Some(rule) = &step.condition_dsl else {
            return true;
        };

        // JSON Logic ルールを評価する（ALG-004）
        JsonLogicEvaluator::new().evaluate(rule, context)
    }

    /// (ALG-003) 並列実行可能なステップを解決する。
    /// 現在は単純な step_number 順を返す（並列 Step は将来拡張）。
    pub fn resolve_parallel_steps(steps: &[Step]) -> Vec<Step> {
        // step_number 順にソートして返す（並列実行は現バージョンでは未対応）
        let mut sorted = steps.to_vec();
        sorted.sort_by_key(|s| s.step_number);
        sorted
    }

    /// (ALG-001) 指定ステップへの前進が可能かどうかを判定する。
    /// BR-BUS-001（ロックステップ）と関連するガード条件を検証する。
    pub fn can_advance_to(
        current_step_index: u32,
        target_step_number: u32,
    ) -> Result<(), DomainError> {
        // ステップスキップ禁止（BR-BUS-001）
        // target は current + 1 でなければならない
        if target_step_number > current_step_index + 1 {
            return Err(DomainError::StepSequenceViolation {
                current_step: current_step_index,
                attempted_step: target_step_number,
            });
        }

        // 完了済みステップへの後退禁止
        if target_step_number <= current_step_index {
            return Err(DomainError::StepSequenceViolation {
                current_step: current_step_index,
                attempted_step: target_step_number,
            });
        }

        Ok(())
    }
}
