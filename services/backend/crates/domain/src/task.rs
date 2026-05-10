//! 「作業（Task）」Aggregate
//!
//! 対応 §: ロードマップ §3.1.1 §3.4.1 §10.1 §28
//!
//! §3.1.1 で定義された 11 構成要素のうち、最初の MVP 範囲として
//! 識別／状態／完了条件／開始条件／主体／時間属性 を表現する。
//! 残りの構成要素（文脈・入出力・例外属性・心理属性・階層）は段階的に追加する（§22）。

// 値オブジェクトの import
use crate::value_object::{
    CompletionCriteria, DeviceId, Evidence, LamportTimestamp, TaskId,
};
// ドメインエラー
use crate::error::DomainError;

/// 作業の状態（[`hsm-task.puml`](../../../../../docs/03_設計/形式化/hsm-task.puml) と整合）
///
/// HSM の主要状態に対応するフラットな列挙型。階層構造は将来拡張で `enum` に
/// バリアントを追加して対応する（型安全な遷移を維持）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// 初期状態（前提条件未充足）
    Idle,
    /// 開始可能（部材・権限・先行作業完了）
    Ready,
    /// 実行中
    Running,
    /// 中断（休憩・割込み）
    Suspended,
    /// 例外（完了条件不適合・設備異常）
    Exception,
    /// 完了
    Completed,
    /// 失敗（タイムアウト等）
    Failed,
    /// 取消（不可逆）
    Aborted,
}

impl TaskState {
    /// 状態を表す文字列リテラルを返す（エラーメッセージ用）
    #[must_use]
    pub const fn label(self) -> &'static str {
        // バリアント名を文字列リテラルに対応付ける
        match self {
            TaskState::Idle => "Idle",
            TaskState::Ready => "Ready",
            TaskState::Running => "Running",
            TaskState::Suspended => "Suspended",
            TaskState::Exception => "Exception",
            TaskState::Completed => "Completed",
            TaskState::Failed => "Failed",
            TaskState::Aborted => "Aborted",
        }
    }

    /// 文字列ラベルから `TaskState` を構築する（永続化からの復元）
    ///
    /// # Errors
    /// 既知でないラベルは `None` を返す（呼び出し側でドメインエラー化する）。
    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        // 既知ラベルを判定する
        match label {
            "Idle" => Some(Self::Idle),
            "Ready" => Some(Self::Ready),
            "Running" => Some(Self::Running),
            "Suspended" => Some(Self::Suspended),
            "Exception" => Some(Self::Exception),
            "Completed" => Some(Self::Completed),
            "Failed" => Some(Self::Failed),
            "Aborted" => Some(Self::Aborted),
            _ => None,
        }
    }
}

/// 作業（Task）Aggregate
///
/// 不変条件:
/// - `state` は `lifecycle()` の遷移グラフに従ってのみ変更される。
/// - `lamport` は単調増加する（INV-08）。
#[derive(Debug, Clone)]
pub struct Task {
    /// 識別（§3.1.1 識別）
    id: TaskId,
    /// 状態（§3.3「状態」観点）
    state: TaskState,
    /// 完了条件（§3.1.1 検証）
    completion_criteria: CompletionCriteria,
    /// 主体としての端末（§3.1.1 主体）
    device_id: DeviceId,
    /// 直近の Lamport クロック値（§10.6.1 INV-08）
    lamport: LamportTimestamp,
    /// 前提条件が満たされているか（§3.1.1 開始条件）
    precondition_satisfied: bool,
}

impl Task {
    /// 新規 Task を Idle 状態で作る
    #[must_use]
    pub fn create(
        id: TaskId,
        completion_criteria: CompletionCriteria,
        device_id: DeviceId,
    ) -> Self {
        // すべての必須要素を設定し、初期状態 Idle で構築する
        Self {
            id,
            state: TaskState::Idle,
            completion_criteria,
            device_id,
            lamport: LamportTimestamp::zero(),
            precondition_satisfied: false,
        }
    }

    /// 永続化された値から Task を **再構築** する（リハイドレート）。
    ///
    /// adapter 層で DB から状態・Lamport・precondition を読み戻すときに使う。
    /// 不変条件の検証は呼び出し側で済ませている前提。
    #[must_use]
    pub const fn rehydrate(
        id: TaskId,
        state: TaskState,
        completion_criteria: CompletionCriteria,
        device_id: DeviceId,
        lamport: LamportTimestamp,
        precondition_satisfied: bool,
    ) -> Self {
        // 永続化値から直接構築する
        Self {
            id,
            state,
            completion_criteria,
            device_id,
            lamport,
            precondition_satisfied,
        }
    }

    /// 完了条件を取得する（adapter 層で永続化に使う）
    #[must_use]
    pub const fn completion_criteria(&self) -> &CompletionCriteria {
        // 借用で返す
        &self.completion_criteria
    }

    /// 前提条件が充足済みか（adapter 層で永続化に使う）
    #[must_use]
    pub const fn precondition_satisfied(&self) -> bool {
        // フラグを返す
        self.precondition_satisfied
    }

    /// 識別子を取得する
    #[must_use]
    pub fn id(&self) -> &TaskId {
        // 内部 TaskId を借用で返す
        &self.id
    }

    /// 状態を取得する
    #[must_use]
    pub const fn state(&self) -> TaskState {
        // バリアントは Copy なのでそのまま返す
        self.state
    }

    /// 主体端末を取得する
    #[must_use]
    pub fn device_id(&self) -> &DeviceId {
        // 内部 DeviceId を借用で返す
        &self.device_id
    }

    /// Lamport クロック値を取得する
    #[must_use]
    pub const fn lamport(&self) -> LamportTimestamp {
        // Copy 型なのでそのまま返す
        self.lamport
    }

    /// 前提条件を満たした旨をマークする（§3.1.1 開始条件）
    pub fn mark_precondition_satisfied(&mut self) -> Result<(), DomainError> {
        // Lamport を進める（端末側のローカルイベント）
        self.lamport = self.lamport.next()?;
        // フラグを立てる
        self.precondition_satisfied = true;
        // Idle → Ready の遷移を内部で行う
        if self.state == TaskState::Idle {
            self.state = TaskState::Ready;
        }
        // 正常終了
        Ok(())
    }

    /// 開始する（Ready → Running）
    ///
    /// # Errors
    /// 開始条件未充足、または不正な状態からの遷移時にエラー。
    pub fn start(&mut self) -> Result<(), DomainError> {
        // 前提条件チェック（§3.1.1）
        if !self.precondition_satisfied {
            return Err(DomainError::PreconditionNotSatisfied);
        }
        // 状態が Ready 以外なら遷移不正
        if self.state != TaskState::Ready {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Running",
            });
        }
        // Lamport を進める
        self.lamport = self.lamport.next()?;
        // 状態を Running に遷移
        self.state = TaskState::Running;
        // 正常終了
        Ok(())
    }

    /// 中断する（Running → Suspended）
    pub fn suspend(&mut self) -> Result<(), DomainError> {
        // 状態が Running 以外なら遷移不正
        if self.state != TaskState::Running {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Suspended",
            });
        }
        // Lamport を進める
        self.lamport = self.lamport.next()?;
        // 状態を Suspended に遷移
        self.state = TaskState::Suspended;
        // 正常終了
        Ok(())
    }

    /// 再開する（Suspended → Running）
    pub fn resume(&mut self) -> Result<(), DomainError> {
        // 状態が Suspended 以外なら遷移不正
        if self.state != TaskState::Suspended {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Running",
            });
        }
        // Lamport を進める
        self.lamport = self.lamport.next()?;
        // 状態を Running に戻す
        self.state = TaskState::Running;
        // 正常終了
        Ok(())
    }

    /// 完了する（Running → Completed、完了条件達成時）
    ///
    /// # Errors
    /// - 状態が Running 以外
    /// - `evidence` が `completion_criteria` を満たさない
    pub fn complete(&mut self, evidence: &Evidence) -> Result<(), DomainError> {
        // 状態が Running 以外なら遷移不正
        if self.state != TaskState::Running {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Completed",
            });
        }
        // 完了条件の判定
        if !self.completion_criteria.is_met(evidence) {
            return Err(DomainError::CompletionCriteriaNotMet);
        }
        // Lamport を進める
        self.lamport = self.lamport.next()?;
        // 状態を Completed に遷移
        self.state = TaskState::Completed;
        // 正常終了
        Ok(())
    }

    /// 取消する（任意の活動状態 → Aborted）
    pub fn abort(&mut self) -> Result<(), DomainError> {
        // 既に Completed／Aborted／Failed の場合は遷移不正
        match self.state {
            TaskState::Completed | TaskState::Aborted | TaskState::Failed => {
                return Err(DomainError::InvalidStateTransition {
                    current: self.state.label(),
                    attempted: "Aborted",
                });
            }
            _ => {}
        }
        // Lamport を進める
        self.lamport = self.lamport.next()?;
        // 状態を Aborted に遷移
        self.state = TaskState::Aborted;
        // 正常終了
        Ok(())
    }
}

// =====================================================================
// 単体テスト（§13.1）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // 補助関数: テスト用の Task を作る
    fn fresh() -> Task {
        // テスト用の TaskId を生成
        let id = TaskId::new("task-1").expect("valid id");
        // テスト用の DeviceId を生成
        let dev = DeviceId::new("device-1").expect("valid id");
        // 完了条件は Manual（最も単純）
        Task::create(id, CompletionCriteria::Manual, dev)
    }

    // 初期状態は Idle で前提未充足
    #[test]
    fn task_starts_in_idle() {
        // テスト用 Task
        let t = fresh();
        // 状態は Idle
        assert_eq!(t.state(), TaskState::Idle);
    }

    // 前提充足で Idle → Ready
    #[test]
    fn precondition_transitions_to_ready() {
        // テスト用 Task
        let mut t = fresh();
        // 前提を満たす
        t.mark_precondition_satisfied().expect("ok");
        // Ready に遷移していること
        assert_eq!(t.state(), TaskState::Ready);
    }

    // start: 前提未充足なら拒否
    #[test]
    fn start_without_precondition_is_rejected() {
        // テスト用 Task
        let mut t = fresh();
        // 即座に start を試みる
        let r = t.start();
        // 前提条件エラーが返る
        assert_eq!(r, Err(DomainError::PreconditionNotSatisfied));
    }

    // start → suspend → resume の経路
    #[test]
    fn start_suspend_resume_cycle() {
        // テスト用 Task
        let mut t = fresh();
        // 前提充足
        t.mark_precondition_satisfied().expect("ok");
        // 開始
        t.start().expect("ok");
        // Running に
        assert_eq!(t.state(), TaskState::Running);
        // 中断
        t.suspend().expect("ok");
        // Suspended に
        assert_eq!(t.state(), TaskState::Suspended);
        // 再開
        t.resume().expect("ok");
        // Running に戻る
        assert_eq!(t.state(), TaskState::Running);
    }

    // complete: 完了条件未充足なら拒否
    #[test]
    fn complete_requires_evidence() {
        // テスト用 Task（Manual 条件）
        let mut t = fresh();
        // 前提充足
        t.mark_precondition_satisfied().expect("ok");
        // 開始
        t.start().expect("ok");
        // 空の証跡で complete
        let r = t.complete(&Evidence::default());
        // 完了条件未充足エラー
        assert_eq!(r, Err(DomainError::CompletionCriteriaNotMet));
    }

    // complete: 証跡があれば完了
    #[test]
    fn complete_succeeds_with_manual_evidence() {
        // テスト用 Task
        let mut t = fresh();
        // 前提充足
        t.mark_precondition_satisfied().expect("ok");
        // 開始
        t.start().expect("ok");
        // 完了条件を満たす証跡
        let ev = Evidence { manually_marked: true, ..Evidence::default() };
        // 完了
        t.complete(&ev).expect("ok");
        // Completed に遷移
        assert_eq!(t.state(), TaskState::Completed);
    }

    // Lamport は遷移ごとに単調増加する（INV-08）
    #[test]
    fn lamport_monotonic_across_transitions() {
        // テスト用 Task
        let mut t = fresh();
        // 初期は 0
        assert_eq!(t.lamport().value(), 0);
        // 前提充足
        t.mark_precondition_satisfied().expect("ok");
        // 開始
        t.start().expect("ok");
        // 中断
        t.suspend().expect("ok");
        // 再開
        t.resume().expect("ok");
        // ここまでで 4 回進んでいるはず
        assert_eq!(t.lamport().value(), 4);
    }

    // mutation testing 検出強化（§13.4.1）

    // abort: Completed からの abort は拒否
    #[test]
    fn abort_rejected_from_completed() {
        let mut t = fresh();
        t.mark_precondition_satisfied().expect("ok");
        t.start().expect("ok");
        let ev = Evidence { manually_marked: true, ..Evidence::default() };
        t.complete(&ev).expect("ok");
        // Completed からの abort はエラー
        let r = t.abort();
        assert!(matches!(r, Err(DomainError::InvalidStateTransition { .. })));
    }

    // abort: 既に Aborted からの abort は拒否
    #[test]
    fn abort_rejected_from_aborted() {
        let mut t = fresh();
        t.mark_precondition_satisfied().expect("ok");
        t.start().expect("ok");
        t.abort().expect("ok");
        // 2 度目はエラー
        let r = t.abort();
        assert!(matches!(r, Err(DomainError::InvalidStateTransition { .. })));
    }

    // abort: Idle/Ready/Running/Suspended/Exception からは成功する
    #[test]
    fn abort_accepted_from_active_states() {
        // Running から
        let mut t1 = fresh();
        t1.mark_precondition_satisfied().expect("ok");
        t1.start().expect("ok");
        assert!(t1.abort().is_ok());
        // Suspended から
        let mut t2 = fresh();
        t2.mark_precondition_satisfied().expect("ok");
        t2.start().expect("ok");
        t2.suspend().expect("ok");
        assert!(t2.abort().is_ok());
    }

    // TaskState::label が値を保つ（mutation 検出）
    #[test]
    fn task_state_label_preserves_value() {
        assert_eq!(TaskState::Idle.label(), "Idle");
        assert_eq!(TaskState::Running.label(), "Running");
        assert_eq!(TaskState::Completed.label(), "Completed");
        assert_eq!(TaskState::Aborted.label(), "Aborted");
        assert_eq!(TaskState::Failed.label(), "Failed");
        assert_eq!(TaskState::Exception.label(), "Exception");
        assert_eq!(TaskState::Ready.label(), "Ready");
        assert_eq!(TaskState::Suspended.label(), "Suspended");
    }
}
